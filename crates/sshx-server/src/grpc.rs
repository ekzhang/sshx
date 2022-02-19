//! Defines gRPC routes and application request logic.

use sshx_core::proto::{sshx_service_server::SshxService, HelloReply, HelloRequest};
use tonic::{Request, Response, Status};

/// Server that handles gRPC requests from the sshx command-line client.
pub struct GrpcServer;

#[tonic::async_trait]
impl SshxService for GrpcServer {
    async fn hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: format!("Hello {}!", request.get_ref().name),
        };

        Ok(Response::new(reply))
    }
}
