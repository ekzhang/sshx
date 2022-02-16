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

use axum::body::HttpBody;
use grpc::GrpcServer;
use hyper::{header::CONTENT_TYPE, service::make_service_fn, Body, Request, Server};
use sshx_core::proto::{greeter_server::GreeterServer, FILE_DESCRIPTOR_SET};
use tonic::transport::Server as TonicServer;
use tower::{steer::Steer, ServiceBuilder, ServiceExt};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{Level, Span};

pub mod grpc;
pub mod http;

/// Make the combined HTTP/gRPC application server, listening on a TCP address.
pub async fn make_server(
    addr: &SocketAddr,
    signal: impl Future<Output = ()>,
) -> anyhow::Result<()> {
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
        .add_service(GreeterServer::new(GrpcServer))
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
            Some(value) if value.to_str().unwrap_or_default() == "application/grpc" => 1,
            _ => 0,
        },
    );
    let make_svc = make_service_fn(|_| {
        let svc = svc.clone();
        async { Ok::<_, std::convert::Infallible>(svc) }
    });

    Server::bind(addr)
        .serve(make_svc)
        .with_graceful_shutdown(signal)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{sync::oneshot, time};
    use tonic::Request;

    use super::make_server;

    #[tokio::test]
    async fn test_rpc() -> Result<(), anyhow::Error> {
        use sshx_core::proto::*;

        let endpoint = "http://[::1]:8051";
        let addr = "[::1]:8051".parse()?;
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            time::sleep(Duration::from_millis(1)).await;
            let req = Request::new(HelloRequest {
                name: "adam".into(),
            });
            let mut client = greeter_client::GreeterClient::connect(endpoint)
                .await
                .unwrap();
            let resp = client.say_hello(req).await.unwrap();
            println!("resp={:?}", resp);

            tx.send(()).unwrap();
        });

        make_server(&addr, async { rx.await.unwrap() }).await?;
        Ok(())
    }
}
