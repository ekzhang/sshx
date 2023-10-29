//! Stateful components of the server, managing multiple sessions.

use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use dashmap::DashMap;
use hmac::{Hmac, Mac as _};
use sha2::Sha256;
use sshx_core::rand_alphanumeric;
use tokio::time;
use tokio_stream::StreamExt;
use tracing::error;

use self::mesh::StorageMesh;
use crate::session::Session;
use crate::ServerOptions;

pub mod mesh;

/// Timeout for a disconnected session to be evicted and closed.
///
/// If a session has no backend clients making connections in this interval,
/// then its updated timestamp will be out-of-date, so we close it and remove it
/// from the state to reduce memory usage.
const DISCONNECTED_SESSION_EXPIRY: Duration = Duration::from_secs(300);

/// Shared state object for global server logic.
pub struct ServerState {
    /// Message authentication code for signing tokens.
    mac: Hmac<Sha256>,

    /// Override the origin returned for the Open() RPC.
    override_origin: Option<String>,

    /// A concurrent map of session IDs to session objects.
    store: DashMap<String, Arc<Session>>,

    /// Storage and distributed communication provider, if enabled.
    mesh: Option<StorageMesh>,
}

impl ServerState {
    /// Create an empty server state using the given secret.
    pub fn new(options: ServerOptions) -> Result<Self> {
        let secret = options.secret.unwrap_or_else(|| rand_alphanumeric(22));
        let mesh = match options.redis_url {
            Some(url) => Some(StorageMesh::new(&url, options.host.as_deref())?),
            None => None,
        };
        Ok(Self {
            mac: Hmac::new_from_slice(secret.as_bytes()).unwrap(),
            override_origin: options.override_origin,
            store: DashMap::new(),
            mesh,
        })
    }

    /// Returns the message authentication code used for signing tokens.
    pub fn mac(&self) -> Hmac<Sha256> {
        self.mac.clone()
    }

    /// Returns the override origin for the Open() RPC.
    pub fn override_origin(&self) -> Option<String> {
        self.override_origin.clone()
    }

    /// Lookup a local session by name.
    pub fn lookup(&self, name: &str) -> Option<Arc<Session>> {
        self.store.get(name).map(|s| s.clone())
    }

    /// Insert a session into the local store.
    pub fn insert(&self, name: &str, session: Arc<Session>) {
        if let Some(mesh) = &self.mesh {
            let name = name.to_string();
            let session = session.clone();
            let mesh = mesh.clone();
            tokio::spawn(async move {
                mesh.background_sync(&name, session).await;
            });
        }
        if let Some(prev_session) = self.store.insert(name.to_string(), session) {
            prev_session.shutdown();
        }
    }

    /// Remove a session from the local store.
    pub fn remove(&self, name: &str) -> bool {
        if let Some((_, session)) = self.store.remove(name) {
            session.shutdown();
            true
        } else {
            false
        }
    }

    /// Close a session permanently on this and other servers.
    pub async fn close_session(&self, name: &str) -> Result<()> {
        self.remove(name);
        if let Some(mesh) = &self.mesh {
            mesh.mark_closed(name).await?;
        }
        Ok(())
    }

    /// Connect to a session by name from the `sshx` client, which provides the
    /// actual terminal backend.
    pub async fn backend_connect(&self, name: &str) -> Result<Option<Arc<Session>>> {
        if let Some(session) = self.lookup(name) {
            return Ok(Some(session));
        }

        if let Some(mesh) = &self.mesh {
            let (owner, snapshot) = mesh.get_owner_snapshot(name).await?;
            if let Some(snapshot) = snapshot {
                let session = Arc::new(Session::restore(&snapshot)?);
                self.insert(name, session.clone());
                if let Some(owner) = owner {
                    mesh.notify_transfer(name, &owner).await?;
                }
                return Ok(Some(session));
            }
        }

        Ok(None)
    }

    /// Connect to a session from a web browser frontend, possibly redirecting.
    pub async fn frontend_connect(
        &self,
        name: &str,
    ) -> Result<Result<Arc<Session>, Option<String>>> {
        if let Some(session) = self.lookup(name) {
            return Ok(Ok(session));
        }

        if let Some(mesh) = &self.mesh {
            let mut owner = mesh.get_owner(name).await?;
            if owner.is_some() && owner.as_deref() == mesh.host() {
                // Do not redirect back to the same server.
                owner = None;
            }
            return Ok(Err(owner));
        }

        Ok(Err(None))
    }

    /// Listen for and remove sessions that are transferred away from this host.
    pub async fn listen_for_transfers(&self) {
        if let Some(mesh) = &self.mesh {
            let mut transfers = pin!(mesh.listen_for_transfers());
            while let Some(name) = transfers.next().await {
                self.remove(&name);
            }
        }
    }

    /// Close all sessions that have been disconnected for too long.
    pub async fn close_old_sessions(&self) {
        loop {
            time::sleep(DISCONNECTED_SESSION_EXPIRY / 5).await;
            let mut to_close = Vec::new();
            for entry in &self.store {
                let session = entry.value();
                if session.last_accessed().elapsed() > DISCONNECTED_SESSION_EXPIRY {
                    to_close.push(entry.key().clone());
                }
            }
            for name in to_close {
                if let Err(err) = self.close_session(&name).await {
                    error!(?err, "failed to close old session {name}");
                }
            }
        }
    }

    /// Send a graceful shutdown signal to every session.
    pub fn shutdown(&self) {
        for entry in &self.store {
            entry.value().shutdown();
        }
    }
}
