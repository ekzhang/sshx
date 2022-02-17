use std::net::{SocketAddr, TcpListener};

use hyper::server::Server;
use sshx_server::make_server;
use tokio::sync::oneshot;

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
    pub fn new() -> anyhow::Result<Self> {
        let listener = TcpListener::bind("[::1]:0")?;
        let local_addr = listener.local_addr()?;

        let (tx, rx) = oneshot::channel();
        let server = make_server(Server::from_tcp(listener)?, async { rx.await.unwrap() });
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

    /// Returns the HTTP/2 gRPC endpoint for this server.
    pub fn endpoint(&self) -> String {
        format!("http://{}", self.local_addr)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.terminate.take().unwrap().send(()).ok();
    }
}
