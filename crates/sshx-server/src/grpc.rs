//! Defines gRPC routes and application request logic.

use sshx_core::proto::{greeter_server::Greeter, HelloReply, HelloRequest};
use tonic::{Request, Response, Status};

/// Server that handles gRPC requests from the sshx command-line client.
pub struct GrpcServer;

#[tonic::async_trait]
impl Greeter for GrpcServer {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: format!("Hello {}!", request.get_ref().name),
        };

        Ok(Response::new(reply))
    }
}
