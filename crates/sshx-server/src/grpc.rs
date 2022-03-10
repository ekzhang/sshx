//! Defines gRPC routes and application request logic.

use sshx_core::proto::{sshx_service_server::SshxService, *};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

/// Server that handles gRPC requests from the sshx command-line client.
pub struct GrpcServer;

#[tonic::async_trait]
impl SshxService for GrpcServer {
    type ChannelStream = ReceiverStream<Result<ServerUpdate, Status>>;

    async fn open(&self, request: Request<OpenRequest>) -> Result<Response<OpenResponse>, Status> {
        let _ = request;
        Ok(Response::new(OpenResponse {
            name: "placeholder".into(),
            url: "https://example.com".into(),
        }))
    }

    async fn channel(
        &self,
        request: Request<Streaming<ClientUpdate>>,
    ) -> Result<Response<Self::ChannelStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(1);

        tokio::spawn(async move {
            let _ = stream.next().await;
            let _ = tx;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn close(
        &self,
        request: Request<CloseRequest>,
    ) -> Result<Response<CloseResponse>, Status> {
        let _ = request;
        todo!()
    }
}
