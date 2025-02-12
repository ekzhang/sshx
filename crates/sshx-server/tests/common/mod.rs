use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{ensure, Result};
use axum::serve::ListenerExt;
use futures_util::{SinkExt, StreamExt};
use http::StatusCode;
use sshx::encrypt::Encrypt;
use sshx_core::proto::sshx_service_client::SshxServiceClient;
use sshx_core::{Sid, Uid};
use sshx_server::{
    state::ServerState,
    web::protocol::{WsClient, WsServer, WsUser, WsWinsize},
    Server,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::time;
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
    pub async fn new() -> Self {
        let listener = TcpListener::bind("[::1]:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let server = Arc::new(Server::new(Default::default()).unwrap());
        {
            let server = Arc::clone(&server);
            let listener = listener.tap_io(|tcp_stream| {
                _ = tcp_stream.set_nodelay(true);
            });
            tokio::spawn(async move {
                server.listen(listener).await.unwrap();
            });
        }

        TestServer { local_addr, server }
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
    pub async fn grpc_client(&self) -> SshxServiceClient<Channel> {
        SshxServiceClient::connect(self.endpoint()).await.unwrap()
    }

    /// Return the current server state object.
    pub fn state(&self) -> Arc<ServerState> {
        self.server.state()
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
    encrypt: Encrypt,
    write_encrypt: Option<Encrypt>,

    pub user_id: Uid,
    pub users: BTreeMap<Uid, WsUser>,
    pub shells: BTreeMap<Sid, WsWinsize>,
    pub data: HashMap<Sid, String>,
    pub messages: Vec<(Uid, String, String)>,
    pub errors: Vec<String>,
}

impl ClientSocket {
    /// Connect to a WebSocket endpoint.
    pub async fn connect(uri: &str, key: &str, write_password: Option<&str>) -> Result<Self> {
        let (stream, resp) = tokio_tungstenite::connect_async(uri).await?;
        ensure!(resp.status() == StatusCode::SWITCHING_PROTOCOLS);

        let mut this = Self {
            inner: stream,
            encrypt: Encrypt::new(key),
            write_encrypt: write_password.map(Encrypt::new),
            user_id: Uid(0),
            users: BTreeMap::new(),
            shells: BTreeMap::new(),
            data: HashMap::new(),
            messages: Vec::new(),
            errors: Vec::new(),
        };
        this.authenticate().await;
        Ok(this)
    }

    async fn authenticate(&mut self) {
        let encrypted_zeros = self.encrypt.zeros().into();
        let write_zeros = self.write_encrypt.as_ref().map(|e| e.zeros().into());

        self.send(WsClient::Authenticate(encrypted_zeros, write_zeros))
            .await;
    }

    pub async fn send(&mut self, msg: WsClient) {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&msg, &mut buf).unwrap();
        self.inner.send(Message::Binary(buf.into())).await.unwrap();
    }

    pub async fn send_input(&mut self, id: Sid, data: &[u8]) {
        let offset = 42; // arbitrary, don't reuse the offset in real code though
        let data = self.encrypt.segment(0x200000000, offset, data);
        self.send(WsClient::Data(id, data.into(), offset)).await;
    }

    async fn recv(&mut self) -> Option<WsServer> {
        loop {
            match self.inner.next().await.transpose().unwrap() {
                Some(Message::Text(_)) => panic!("unexpected text message over WebSocket"),
                Some(Message::Binary(msg)) => {
                    break Some(ciborium::de::from_reader(&*msg).unwrap())
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

    pub async fn flush(&mut self) {
        const FLUSH_DURATION: Duration = Duration::from_millis(50);
        let flush_task = async {
            while let Some(msg) = self.recv().await {
                match msg {
                    WsServer::Hello(user_id, _) => self.user_id = user_id,
                    WsServer::InvalidAuth() => panic!("invalid authentication"),
                    WsServer::Users(users) => self.users = BTreeMap::from_iter(users),
                    WsServer::UserDiff(id, maybe_user) => {
                        self.users.remove(&id);
                        if let Some(user) = maybe_user {
                            self.users.insert(id, user);
                        }
                    }
                    WsServer::Shells(shells) => self.shells = BTreeMap::from_iter(shells),
                    WsServer::Chunks(id, seqnum, chunks) => {
                        let value = self.data.entry(id).or_default();
                        assert_eq!(seqnum, value.len() as u64);
                        for buf in chunks {
                            let plaintext = self.encrypt.segment(
                                0x100000000 | id.0 as u64,
                                value.len() as u64,
                                &buf,
                            );
                            value.push_str(std::str::from_utf8(&plaintext).unwrap());
                        }
                    }
                    WsServer::Hear(id, name, msg) => {
                        self.messages.push((id, name, msg));
                    }
                    WsServer::ShellLatency(_) => {}
                    WsServer::Pong(_) => {}
                    WsServer::Error(err) => self.errors.push(err),
                }
            }
        };
        time::timeout(FLUSH_DURATION, flush_task).await.ok();
    }

    pub fn read(&self, id: Sid) -> &str {
        self.data.get(&id).map(|s| &**s).unwrap_or("")
    }
}
