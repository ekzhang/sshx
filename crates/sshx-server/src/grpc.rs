//! Defines gRPC routes and application request logic.

use std::{sync::Arc, time::Duration};

use nanoid::nanoid;
use sshx_core::proto::{
    client_update::ClientMessage, server_update::ServerMessage, sshx_service_server::SshxService,
    ClientUpdate, CloseRequest, CloseResponse, OpenRequest, OpenResponse, SequenceNumbers,
    ServerUpdate,
};
use tokio::{sync::mpsc, time};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::{error, info, warn};

use crate::session::{Session, SessionStore};

/// Interval for synchronizing sequence numbers from the server.
pub const SYNC_INTERVAL: Duration = Duration::from_secs(5);

/// Server that handles gRPC requests from the sshx command-line client.
#[derive(Clone)]
pub struct GrpcServer(SessionStore);

impl GrpcServer {
    /// Construct a new [`GrpcServer`] instance with associated state.
    pub fn new(store: SessionStore) -> Self {
        Self(store)
    }
}

#[tonic::async_trait]
impl SshxService for GrpcServer {
    type ChannelStream = ReceiverStream<Result<ServerUpdate, Status>>;

    async fn open(&self, request: Request<OpenRequest>) -> Result<Response<OpenResponse>, Status> {
        use dashmap::mapref::entry::Entry::*;

        let domain = request.into_inner().domain;
        if domain.is_empty() {
            return Err(Status::invalid_argument("domain is empty"));
        }
        let id = nanoid!();
        info!(%id, "creating new session");
        match self.0.entry(id.clone()) {
            Occupied(_) => return Err(Status::already_exists("generated duplicate ID")),
            Vacant(v) => v.insert(Session::new().into()),
        };
        Ok(Response::new(OpenResponse {
            name: id.clone(),
            url: format!("https://{domain}/join/{id}"),
        }))
    }

    async fn channel(
        &self,
        request: Request<Streaming<ClientUpdate>>,
    ) -> Result<Response<Self::ChannelStream>, Status> {
        let mut stream = request.into_inner();
        let first_update = match stream.next().await {
            Some(result) => result?,
            None => return Err(Status::invalid_argument("missing first message")),
        };
        let session_name = match client_msg(first_update)? {
            ClientMessage::SessionName(name) => name,
            _ => return Err(Status::invalid_argument("invalid first message")),
        };
        let session = match self.0.get(&session_name) {
            Some(session) => Arc::clone(&session),
            None => return Err(Status::not_found("session not found")),
        };

        // We now spawn an asynchronous task that sends updates to the client. Note that
        // when this task finishes, the sender end is dropped, so the receiver is
        // automatically closed.
        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            let mut interval = time::interval(SYNC_INTERVAL);
            loop {
                let msg = tokio::select! {
                    // Send periodic sync messages to the server.
                    _ = interval.tick() => {
                        let map = session.sequence_numbers();
                        let msg = ServerMessage::Sync(SequenceNumbers { map });
                        if send_msg(&tx, msg).await {
                            continue;
                        } else {
                            break;
                        }
                    }
                    // Handle incoming client messages.
                    maybe_update = stream.next() => {
                        if let Some(Ok(update)) = maybe_update {
                            match client_msg(update) {
                                Ok(msg) => msg,
                                Err(err) => {
                                    let _ = tx.send(Err(err)).await;
                                    break;
                                }
                            }
                        } else {
                            // The client has hung up on their end.
                            return;
                        }
                    }
                };

                match msg {
                    ClientMessage::SessionName(_) => {
                        if !send_err(&tx, "unexpected session name".into()).await {
                            break;
                        }
                    }
                    ClientMessage::Data(data) => {
                        if let Err(err) = session.add_data(data.id, &data.data, data.seq) {
                            if !send_err(&tx, format!("add data: {:?}", err)).await {
                                break;
                            }
                        }
                    }
                    ClientMessage::CreatedShell(id) => {
                        if let Err(err) = session.add_shell(id) {
                            if !send_err(&tx, format!("add shell: {:?}", err)).await {
                                break;
                            }
                        }
                    }
                    ClientMessage::ClosedShell(id) => {
                        if let Err(err) = session.close_shell(id) {
                            if !send_err(&tx, format!("close shell: {:?}", err)).await {
                                break;
                            }
                        }
                    }
                    ClientMessage::Error(err) => {
                        error!(?err, "error received from client");
                    }
                }
            }
            warn!("connection exiting early due to an error");
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn close(
        &self,
        request: Request<CloseRequest>,
    ) -> Result<Response<CloseResponse>, Status> {
        let _ = request;
        todo!()
    }
}

/// Extracts the client message enum from an update.
fn client_msg(update: ClientUpdate) -> Result<ClientMessage, Status> {
    update
        .client_message
        .ok_or_else(|| Status::invalid_argument("message is missing from client update"))
}

/// Attempt to send a server message to the client.
async fn send_msg(tx: &mpsc::Sender<Result<ServerUpdate, Status>>, message: ServerMessage) -> bool {
    let update = Ok(ServerUpdate {
        server_message: Some(message),
    });
    tx.send(update).await.is_ok()
}

/// Attempt to send an error message to the client.
async fn send_err(tx: &mpsc::Sender<Result<ServerUpdate, Status>>, err: String) -> bool {
    send_msg(tx, ServerMessage::Error(err)).await
}
