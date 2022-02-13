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

pub mod grpc;

#[cfg(test)]
mod tests {
    use tonic::Request;

    #[tokio::test]
    async fn test_rpc() -> Result<(), Box<dyn std::error::Error>> {
        use sshx_core::proto::*;

        let req = Request::new(HelloRequest {
            name: "adam".into(),
        });
        let _ = req;
        // let mut client = greeter_client::GreeterClient::connect("http://[::1]:8051").await?;
        // let resp = client.say_hello(req).await?;
        // println!("resp={:?}", resp);
        Ok(())
    }
}
