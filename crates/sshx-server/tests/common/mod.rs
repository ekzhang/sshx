use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use hyper::server::conn::AddrIncoming;
use sshx_core::proto::sshx_service_client::SshxServiceClient;
use sshx_server::session::Session;
use sshx_server::state::ServerState;
use sshx_server::Server;
use tokio::net::TcpListener;
use tonic::transport::Channel;

/// An ephemeral, isolated server that is created for each test.
pub struct TestServer {
    local_addr: SocketAddr,
    server: Arc<Server>,
}

impl TestServer {
    /// Create a fresh server listening on an unused local port for testing.
    ///
    /// Returns an object with the local address, as well as a custom [`Drop`]
    /// implementation that gracefully shuts down the server.
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("[::1]:0").await?;
        let local_addr = listener.local_addr()?;

        let incoming = AddrIncoming::from_listener(listener)?;
        let server = Arc::new(Server::new());
        {
            let server = Arc::clone(&server);
            tokio::spawn(async move {
                server.listen(incoming).await.unwrap();
            });
        }

        Ok(TestServer { local_addr, server })
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

    /// Return the current server state object.
    pub fn state(&self) -> Arc<ServerState> {
        self.server.state()
    }

    /// Returns the session associated with the given controller name.
    pub fn find_session(&self, name: &str) -> Option<Arc<Session>> {
        self.state().store.get(name).map(|s| s.clone())
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.server.shutdown();
    }
}
