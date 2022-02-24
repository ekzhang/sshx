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

use std::{error::Error as StdError, future::Future, net::SocketAddr};

use anyhow::Result;
use axum::body::HttpBody;
use grpc::GrpcServer;
use hyper::{
    header::CONTENT_TYPE,
    server::{conn::AddrIncoming, Builder, Server},
    service::make_service_fn,
    Body, Request,
};
use sshx_core::proto::{sshx_service_server::SshxServiceServer, FILE_DESCRIPTOR_SET};
use tonic::transport::Server as TonicServer;
use tower::{steer::Steer, ServiceBuilder, ServiceExt};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{Level, Span};

pub mod grpc;
pub mod http;

/// Make the combined HTTP/gRPC application server, on a given listener.
pub async fn make_server(
    builder: Builder<AddrIncoming>,
    signal: impl Future<Output = ()>,
) -> Result<()> {
    type BoxError = Box<dyn StdError + Send + Sync>;

    let http_service = http::app()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new())
                .on_request(|request: &Request<Body>, _span: &Span| {
                    tracing::info!("started HTTP {} {}", request.method(), request.uri().path())
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .map_response(|r| r.map(|b| b.map_err(BoxError::from).boxed_unsync()))
        .map_err(BoxError::from)
        .boxed_clone();

    let grpc_service = TonicServer::builder()
        .add_service(SshxServiceServer::new(GrpcServer))
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()?,
        )
        .into_service();

    let grpc_service = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_grpc()
                .make_span_with(DefaultMakeSpan::new())
                .on_request(|request: &Request<Body>, _span: &Span| {
                    tracing::info!("started gRPC {}", request.uri().path())
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .service(grpc_service)
        .map_response(|r| r.map(|b| b.map_err(BoxError::from).boxed_unsync()))
        .boxed_clone();

    let svc = Steer::new(
        [http_service, grpc_service],
        |req: &Request<Body>, _services: &[_]| match req.headers().get(CONTENT_TYPE) {
            Some(value) if value.to_str().ok() == Some("application/grpc") => 1,
            _ => 0,
        },
    );
    let make_svc = make_service_fn(move |_| {
        let svc = svc.clone();
        async { Ok::<_, std::convert::Infallible>(svc) }
    });

    builder
        .serve(make_svc)
        .with_graceful_shutdown(signal)
        .await?;

    Ok(())
}

/// Convenience function to call [`make_server`] bound to a TCP address.
pub async fn make_server_bind(addr: &SocketAddr, signal: impl Future<Output = ()>) -> Result<()> {
    make_server(Server::try_bind(addr)?, signal).await
}
