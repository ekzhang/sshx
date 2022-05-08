//! Defines gRPC routes and application request logic.

use nanoid::nanoid;
use sshx_core::proto::{sshx_service_server::SshxService, *};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

use crate::session::{Session, SessionStore};

/// Server that handles gRPC requests from the sshx command-line client.
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
        let (tx, rx) = mpsc::channel(1);

        tokio::spawn(async move {
            let _ = stream.next().await;
            let _ = tx;
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
