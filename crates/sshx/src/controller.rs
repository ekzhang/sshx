//! Network gRPC client allowing server control of terminals.

use anyhow::Result;
use sshx_core::proto::{sshx_service_client::SshxServiceClient, CloseRequest, OpenRequest};
use tonic::transport::Channel;
use tracing::info;

/// Handles a singel session's communication with the remote server.
pub struct Controller {
    client: SshxServiceClient<Channel>,
    name: String,
    token: String,
}

impl Controller {
    /// Construct a new controller, connecting to the remote server.
    pub async fn new(origin: &str) -> Result<Self> {
        info!(%origin, "connecting to server");
        let mut client = SshxServiceClient::connect(String::from(origin)).await?;
        let req = OpenRequest {
            origin: origin.into(),
        };
        let resp = client.open(req).await?.into_inner();
        info!(url = %resp.url, "opened new session");
        Ok(Self {
            client,
            name: resp.name,
            token: resp.token,
        })
    }

    /// Terminate this session gracefully.
    pub async fn close(&self) -> Result<bool> {
        info!("closing session");
        let req = CloseRequest {
            name: self.name.clone(),
            token: self.token.clone(),
        };
        let resp = self.client.clone().close(req).await?.into_inner();
        Ok(resp.exists)
    }
}
