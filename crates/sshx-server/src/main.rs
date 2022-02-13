use sshx_core::proto::{greeter_server::GreeterServer, FILE_DESCRIPTOR_SET};
use sshx_server::grpc::GrpcServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let grpc_service = GreeterServer::new(GrpcServer);
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let addr = "[::1]:8051".parse()?;
    Server::builder()
        .add_service(grpc_service)
        .add_service(reflection_service)
        .serve(addr)
        .await?;
    Ok(())
}
