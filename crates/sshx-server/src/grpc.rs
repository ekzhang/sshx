//! Defines gRPC routes and application request logic.

use std::sync::Arc;
use std::time::Duration;

use base64::prelude::{Engine as _, BASE64_STANDARD};
use hmac::Mac;
use sshx_core::proto::{
    client_update::ClientMessage, server_update::ServerMessage, sshx_service_server::SshxService,
    ClientUpdate, CloseRequest, CloseResponse, OpenRequest, OpenResponse, ServerUpdate,
};
use sshx_core::{rand_alphanumeric, Sid};
use tokio::sync::mpsc;
use tokio::time::{self, MissedTickBehavior};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::{error, info, warn};

use crate::session::{Metadata, Session};
use crate::ServerState;

/// Interval for synchronizing sequence numbers with the client.
pub const SYNC_INTERVAL: Duration = Duration::from_secs(5);

/// Server that handles gRPC requests from the sshx command-line client.
#[derive(Clone)]
pub struct GrpcServer(Arc<ServerState>);

impl GrpcServer {
    /// Construct a new [`GrpcServer`] instance with associated state.
    pub fn new(state: Arc<ServerState>) -> Self {
        Self(state)
    }
}

type RR<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl SshxService for GrpcServer {
    type ChannelStream = ReceiverStream<Result<ServerUpdate, Status>>;

    async fn open(&self, request: Request<OpenRequest>) -> RR<OpenResponse> {
        let request = request.into_inner();
        let origin = self.0.override_origin().unwrap_or(request.origin);
        if origin.is_empty() {
            return Err(Status::invalid_argument("origin is empty"));
        }
        let name = rand_alphanumeric(10);
        info!(%name, "creating new session");
        match self.0.lookup(&name) {
            Some(_) => return Err(Status::already_exists("generated duplicate ID")),
            None => {
                let metadata = Metadata {
                    encrypted_zeros: request.encrypted_zeros,
                };
                self.0.insert(&name, Arc::new(Session::new(metadata)));
            }
        };
        let token = self.0.mac().chain_update(&name).finalize();
        let url = format!("{origin}/s/{name}");
        Ok(Response::new(OpenResponse {
            name,
            token: BASE64_STANDARD.encode(token.into_bytes()),
            url,
        }))
    }

    async fn channel(&self, request: Request<Streaming<ClientUpdate>>) -> RR<Self::ChannelStream> {
        let mut stream = request.into_inner();
        let first_update = match stream.next().await {
            Some(result) => result?,
            None => return Err(Status::invalid_argument("missing first message")),
        };
        let session_name = match first_update.client_message {
            Some(ClientMessage::Hello(hello)) => {
                let (name, token) = hello
                    .split_once(',')
                    .ok_or_else(|| Status::invalid_argument("missing name and token"))?;
                validate_token(self.0.mac(), name, token)?;
                name.to_string()
            }
            _ => return Err(Status::invalid_argument("invalid first message")),
        };
        let session = match self.0.backend_connect(&session_name).await {
            Ok(Some(session)) => session,
            Ok(None) => return Err(Status::not_found("session not found")),
            Err(err) => {
                error!(?err, "failed to connect to backend session");
                return Err(Status::internal(err.to_string()));
            }
        };

        // We now spawn an asynchronous task that sends updates to the client. Note that
        // when this task finishes, the sender end is dropped, so the receiver is
        // automatically closed.
        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            if let Err(err) = handle_streaming(&tx, &session, stream).await {
                warn!(?err, "connection exiting early due to an error");
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn close(&self, request: Request<CloseRequest>) -> RR<CloseResponse> {
        let request = request.into_inner();
        validate_token(self.0.mac(), &request.name, &request.token)?;
        if let Err(err) = self.0.close_session(&request.name).await {
            error!(?err, "failed to close session");
            return Err(Status::internal(err.to_string()));
        }
        Ok(Response::new(CloseResponse {}))
    }
}

/// Validate the client token for a session.
fn validate_token(mac: impl Mac, name: &str, token: &str) -> Result<(), Status> {
    if let Ok(token) = BASE64_STANDARD.decode(token) {
        if mac.chain_update(name).verify_slice(&token).is_ok() {
            return Ok(());
        }
    }
    Err(Status::unauthenticated("invalid token"))
}

type ServerTx = mpsc::Sender<Result<ServerUpdate, Status>>;

/// Handle bidirectional streaming messages RPC messages.
async fn handle_streaming(
    tx: &ServerTx,
    session: &Session,
    mut stream: Streaming<ClientUpdate>,
) -> Result<(), &'static str> {
    let mut interval = time::interval(SYNC_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            // Send periodic sync messages to the client.
            _ = interval.tick() => {
                let msg = ServerMessage::Sync(session.sequence_numbers());
                if !send_msg(tx, msg).await {
                    return Err("failed to send sync message");
                }
            }
            // Send buffered server updates to the client.
            Ok(msg) = session.update_rx().recv() => {
                if !send_msg(tx, msg).await {
                    return Err("failed to send update message");
                }
            }
            // Handle incoming client messages.
            maybe_update = stream.next() => {
                if let Some(Ok(update)) = maybe_update {
                    if !handle_update(tx, session, update).await {
                        return Err("error responding to client update");
                    }
                } else {
                    // The client has hung up on their end.
                    return Ok(());
                }
            }
            // Exit on a session shutdown signal.
            _ = session.terminated() => {
                let msg = String::from("disconnecting because session is closed");
                send_msg(tx, ServerMessage::Error(msg)).await;
                return Ok(());
            }
        };
    }
}

/// Handles a singe update from the client. Returns `true` on success.
async fn handle_update(tx: &ServerTx, session: &Session, update: ClientUpdate) -> bool {
    session.access();
    match update.client_message {
        Some(ClientMessage::Hello(_)) => {
            return send_err(tx, "unexpected hello".into()).await;
        }
        Some(ClientMessage::Data(data)) => {
            if let Err(err) = session.add_data(Sid(data.id), data.data, data.seq) {
                return send_err(tx, format!("add data: {:?}", err)).await;
            }
        }
        Some(ClientMessage::CreatedShell(new_shell)) => {
            let id = Sid(new_shell.id);
            let center = (new_shell.x, new_shell.y);
            if let Err(err) = session.add_shell(id, center) {
                return send_err(tx, format!("add shell: {:?}", err)).await;
            }
        }
        Some(ClientMessage::ClosedShell(id)) => {
            if let Err(err) = session.close_shell(Sid(id)) {
                return send_err(tx, format!("close shell: {:?}", err)).await;
            }
        }
        Some(ClientMessage::Error(err)) => {
            // TODO: Propagate these errors to listeners on the web interface?
            error!(?err, "error received from client");
        }
        None => (), // Heartbeat message, ignored.
    }
    true
}

/// Attempt to send a server message to the client.
async fn send_msg(tx: &ServerTx, message: ServerMessage) -> bool {
    let update = Ok(ServerUpdate {
        server_message: Some(message),
    });
    tx.send(update).await.is_ok()
}

/// Attempt to send an error string to the client.
async fn send_err(tx: &ServerTx, err: String) -> bool {
    send_msg(tx, ServerMessage::Error(err)).await
}
