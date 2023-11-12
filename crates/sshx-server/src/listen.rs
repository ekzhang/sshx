use std::{error::Error as StdError, future::Future, sync::Arc};

use anyhow::Result;
use axum::body::HttpBody;
use hyper::{
    header::CONTENT_TYPE,
    server::{conn::AddrIncoming, Server as HyperServer},
    service::make_service_fn,
    Body, Request,
};
use sshx_core::proto::{sshx_service_server::SshxServiceServer, FILE_DESCRIPTOR_SET};
use tonic::transport::Server as TonicServer;
use tower::{steer::Steer, ServiceBuilder, ServiceExt};
use tower_http::trace::TraceLayer;

use crate::{grpc::GrpcServer, web, ServerState};

/// Bind and listen from the application, with a state and termination signal.
///
/// This internal method is responsible for multiplexing the HTTP and gRPC
/// servers onto a single, consolidated `hyper` service.
pub(crate) async fn start_server(
    state: Arc<ServerState>,
    incoming: AddrIncoming,
    signal: impl Future<Output = ()>,
) -> Result<()> {
    type BoxError = Box<dyn StdError + Send + Sync>;

    let http_service = web::app()
        .with_state(state.clone())
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
    let svc = ServiceBuilder::new().buffer(8).service(svc).boxed_clone();

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
