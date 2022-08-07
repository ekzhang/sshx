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

use std::{error::Error as StdError, future::Future, net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::body::HttpBody;
use grpc::GrpcServer;
use hyper::{
    header::CONTENT_TYPE,
    server::{conn::AddrIncoming, Server as HyperServer},
    service::make_service_fn,
    Body, Request,
};
use nanoid::nanoid;
use sshx_core::proto::{sshx_service_server::SshxServiceServer, FILE_DESCRIPTOR_SET};
use tonic::transport::Server as TonicServer;
use tower::{steer::Steer, ServiceBuilder, ServiceExt};
use tower_http::trace::TraceLayer;
use utils::Shutdown;

use crate::state::ServerState;

pub mod grpc;
pub mod session;
pub mod state;
pub mod utils;
pub mod web;

/// The combined HTTP/gRPC application server for sshx.
pub struct Server {
    state: Arc<ServerState>,
    shutdown: Shutdown,
}

impl Server {
    /// Create a new application server, but do not listen for connections yet.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let secret = nanoid!();
        Self {
            state: Arc::new(ServerState::new(&secret)),
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
        make_server(Arc::clone(&self.state), incoming, self.terminated()).await
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

/// Make the application server, with a given state and termination signal.
async fn make_server(
    state: Arc<ServerState>,
    incoming: AddrIncoming,
    signal: impl Future<Output = ()>,
) -> Result<()> {
    type BoxError = Box<dyn StdError + Send + Sync>;

    let http_service = web::app(state.clone())
        .layer(TraceLayer::new_for_http())
        .map_response(|r| r.map(|b| b.map_err(BoxError::from).boxed_unsync()))
        .map_err(BoxError::from)
        .boxed_clone();

    let grpc_service = TonicServer::builder()
        .add_service(SshxServiceServer::new(GrpcServer::new(state)))
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()?,
        )
        .into_service();

    let grpc_service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc())
        .service(grpc_service)
        .map_response(|r| r.map(|b| b.map_err(BoxError::from).boxed_unsync()))
        .boxed_clone();

    let svc = Steer::new(
        [http_service, grpc_service],
        |req: &Request<Body>, _services: &[_]| {
            let headers = req.headers();
            match headers.get(CONTENT_TYPE) {
                Some(content) if content == "application/grpc" => 1,
                _ => 0,
            }
        },
    );
    let make_svc = make_service_fn(move |_| {
        let svc = svc.clone();
        async { Ok::<_, std::convert::Infallible>(svc) }
    });

    HyperServer::builder(incoming)
        .tcp_nodelay(true)
        .serve(make_svc)
        .with_graceful_shutdown(signal)
        .await?;

    Ok(())
}
