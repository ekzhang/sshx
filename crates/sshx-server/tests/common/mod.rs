use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{ensure, Result};
use futures_util::{SinkExt, StreamExt};
use hyper::{server::conn::AddrIncoming, StatusCode};
use sshx_core::proto::sshx_service_client::SshxServiceClient;
use sshx_server::state::ServerState;
use sshx_server::Server;
use sshx_server::{session::Session, web::WsClient};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
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

    /// Returns the WebSocket endpoint for streaming connections to a session.
    pub fn ws_endpoint(&self, name: &str) -> String {
        format!("ws://{}/api/s/{}", self.local_addr, name)
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

/// A WebSocket client that interacts with the server, used for testing.
pub struct ClientSocket {
    inner: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl ClientSocket {
    /// Connect to a WebSocket endpoint.
    pub async fn connect(uri: &str) -> Result<Self> {
        let (stream, resp) = tokio_tungstenite::connect_async(uri).await?;
        ensure!(resp.status() == StatusCode::SWITCHING_PROTOCOLS);
        Ok(Self { inner: stream })
    }

    pub async fn send(&mut self, msg: WsClient) {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&msg, &mut buf).unwrap();
        self.inner.send(Message::Binary(buf)).await.unwrap();
    }

    pub async fn recv(&mut self) -> Option<WsClient> {
        loop {
            match self.inner.next().await.transpose().unwrap() {
                Some(Message::Text(_)) => panic!("unexpected text message over WebSocket"),
                Some(Message::Binary(msg)) => {
                    break Some(ciborium::de::from_reader(&msg[..]).unwrap())
                }
                Some(_) => (), // ignore other message types, keep looping
                None => break None,
            }
        }
    }

    pub async fn expect_close(&mut self, code: u16) {
        let msg = self.inner.next().await.unwrap().unwrap();
        match msg {
            Message::Close(Some(frame)) => assert!(frame.code == code.into()),
            _ => panic!("unexpected non-close message over WebSocket: {:?}", msg),
        }
    }
}
