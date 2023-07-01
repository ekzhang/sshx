//! The sshx server, which coordinates terminal sharing.
//!
//! Requests are communicated to the server via gRPC (for command-line sharing
//! clients) and WebSocket connections (for web listeners). The server is built
//! using a hybrid Hyper service, split between a Tonic gRPC handler and an Axum
//! web listener.
//!
//! Most web requests are routed directly to static files located in the `dist/`
//! folder relative to where this binary is running, allowing the frontend to be
//! separately developed from the server.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use dashmap::DashMap;
use hmac::{Hmac, Mac as _};
use hyper::server::conn::AddrIncoming;
use nanoid::nanoid;
use session::Session;
use sha2::Sha256;
use utils::Shutdown;

pub mod grpc;
mod listen;
pub mod session;
pub mod utils;
pub mod web;

/// Options when constructing the application server.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ServerOptions {
    /// Secret used for signing tokens. Set randomly if not provided.
    pub secret: Option<String>,

    /// Override the origin returned for the Open() RPC.
    pub override_origin: Option<String>,
}

/// Shared state object for global server logic.
pub struct ServerState {
    /// Message authentication code for signing tokens.
    pub mac: Hmac<Sha256>,

    /// Override the origin returned for the Open() RPC.
    pub override_origin: Option<String>,

    /// A concurrent map of session IDs to session objects.
    pub store: DashMap<String, Arc<Session>>,
}

impl ServerState {
    /// Create an empty server state using the given secret.
    pub fn new(options: ServerOptions) -> Self {
        let secret = options.secret.unwrap_or_else(|| nanoid!());
        Self {
            mac: Hmac::new_from_slice(secret.as_bytes()).unwrap(),
            override_origin: options.override_origin,
            store: DashMap::new(),
        }
    }
}

/// Stateful object that manages the sshx server, with graceful termination.
pub struct Server {
    state: Arc<ServerState>,
    shutdown: Shutdown,
}

impl Server {
    /// Create a new application server, but do not listen for connections yet.
    pub fn new(options: ServerOptions) -> Self {
        Self {
            state: Arc::new(ServerState::new(options)),
            shutdown: Shutdown::new(),
        }
    }

    /// Returns the server's state object.
    pub fn state(&self) -> Arc<ServerState> {
        Arc::clone(&self.state)
    }

    /// Returns a future that resolves when the server is terminated.
    async fn terminated(&self) {
        self.shutdown.wait().await
    }

    /// Run the application server, listening on a stream of connections.
    pub async fn listen(&self, incoming: AddrIncoming) -> Result<()> {
        listen::start_server(self.state(), incoming, self.terminated()).await
    }

    /// Convenience function to call [`Server::listen`] bound to a TCP address.
    pub async fn bind(&self, addr: &SocketAddr) -> Result<()> {
        self.listen(AddrIncoming::bind(addr)?).await
    }

    /// Send a graceful shutdown signal to the server.
    pub fn shutdown(&self) {
        // Stop receiving new network connections.
        self.shutdown.shutdown();
        // Terminate each of the existing sessions.
        for entry in &self.state.store {
            entry.value().shutdown();
        }
    }
}
