//! The sshx server, which coordinates terminal sharing.
//!
//! Requests are communicated to the server via gRPC (for command-line sharing
//! clients) and WebSocket connections (for web listeners). The server is built
//! using a hybrid Hyper service, split between a Tonic gRPC handler and an Axum
//! web listener.
//!
//! Most web requests are routed directly to static files located in the
//! `build/` folder relative to where this binary is running, allowing the
//! frontend to be separately developed from the server.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::serve::{Listener, ListenerExt};
use tokio::net::TcpListener;
use tracing::debug;
use utils::Shutdown;

use crate::state::ServerState;

pub mod grpc;
mod listen;
pub mod session;
pub mod state;
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

    /// URL of the Redis server that stores session data.
    pub redis_url: Option<String>,

    /// Hostname of this server, if running multiple servers.
    pub host: Option<String>,
}

/// Stateful object that manages the sshx server, with graceful termination.
pub struct Server {
    state: Arc<ServerState>,
    shutdown: Shutdown,
}

impl Server {
    /// Create a new application server, but do not listen for connections yet.
    pub fn new(options: ServerOptions) -> Result<Self> {
        Ok(Self {
            state: Arc::new(ServerState::new(options)?),
            shutdown: Shutdown::new(),
        })
    }

    /// Returns the server's state object.
    pub fn state(&self) -> Arc<ServerState> {
        Arc::clone(&self.state)
    }

    /// Run the application server, listening on a stream of connections.
    pub async fn listen<L>(&self, listener: L) -> Result<()>
    where
        L: Listener,
        L::Addr: Debug,
    {
        let state = self.state.clone();
        let terminated = self.shutdown.wait();
        tokio::spawn(async move {
            let background_tasks = futures_util::future::join(
                state.listen_for_transfers(),
                state.close_old_sessions(),
            );
            tokio::select! {
                _ = terminated => {}
                _ = background_tasks => {}
            }
        });

        listen::start_server(self.state(), listener, self.shutdown.wait()).await
    }

    /// Convenience function to call [`Server::listen`] bound to a TCP address.
    ///
    /// This also sets `TCP_NODELAY` on the incoming connections for performance
    /// reasons, as a reasonable default.
    pub async fn bind(&self, addr: &SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?.tap_io(|tcp_stream| {
            if let Err(err) = tcp_stream.set_nodelay(true) {
                debug!("failed to set TCP_NODELAY on incoming connection: {err:#}");
            }
        });
        self.listen(listener).await
    }

    /// Send a graceful shutdown signal to the server.
    pub fn shutdown(&self) {
        // Stop receiving new network connections.
        self.shutdown.shutdown();
        // Terminate each of the existing sessions.
        self.state.shutdown();
    }
}
