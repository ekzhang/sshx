use std::net::SocketAddr;

use anyhow::Result;
use hyper::server::conn::AddrIncoming;
use sshx_core::proto::sshx_service_client::SshxServiceClient;
use sshx_server::make_server;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tonic::transport::Channel;

/// An ephemeral, isolated server that is created for each test.
pub struct TestServer {
    local_addr: SocketAddr,
    terminate: Option<oneshot::Sender<()>>,
}

impl TestServer {
    /// Create a fresh server listening on an unused local port for testing.
    ///
    /// Returns an object with the local address, as well as a custom [`Drop`]
    /// implementation that gracefully shuts down the server.
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("[::1]:0").await?;
        let local_addr = listener.local_addr()?;

        let (tx, rx) = oneshot::channel();
        let incoming = AddrIncoming::from_listener(listener)?;
        let server = make_server(incoming, async { rx.await.unwrap() });
        tokio::spawn(async move {
            server.await.unwrap();
        });

        Ok(TestServer {
            local_addr,
            terminate: Some(tx),
        })
    }

    /// Returns the local TCP address of this server.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Returns the HTTP/2 base endpoint URI for this server.
    pub fn endpoint(&self) -> String {
        format!("http://{}", self.local_addr)
    }

    /// Creates a gRPC client connected to this server.
    pub async fn grpc_client(&self) -> Result<SshxServiceClient<Channel>> {
        Ok(SshxServiceClient::connect(self.endpoint()).await?)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.terminate.take().unwrap().send(()).ok();
    }
}
