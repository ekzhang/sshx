//! Network gRPC client allowing server control of terminals.

use anyhow::{bail, Context, Result};
use sshx_core::proto::client_update::ClientMessage;
use sshx_core::proto::server_update::ServerMessage;
use sshx_core::proto::ClientUpdate;
use sshx_core::proto::{sshx_service_client::SshxServiceClient, CloseRequest, OpenRequest};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tracing::{error, info};

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

    /// Run the controller forever, listening for requests from the server.
    pub async fn run(&self) -> ! {
        loop {
            if let Err(err) = self.try_channel().await {
                error!(?err, "disconnected, retrying in 1s...");
                time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    /// Helper function used by `run()` that can return errors.
    async fn try_channel(&self) -> Result<()> {
        let (tx, rx) = mpsc::channel(16);
        let resp = self.client.clone().channel(ReceiverStream::new(rx)).await?;
        let mut messages = resp.into_inner(); // A stream of server messages.

        let hello = ClientMessage::Hello(format!("{},{}", self.name, self.token));
        send_msg(&tx, hello).await.context("error during Hello")?;

        loop {
            let message = match messages.next().await {
                Some(result) => result
                    .context("error from server")?
                    .server_message
                    .context("message from server is missing")?,
                None => bail!("server closed connection"),
            };

            match message {
                ServerMessage::Data(_) => todo!(),
                ServerMessage::CreateShell(_) => todo!(),
                ServerMessage::CloseShell(_) => todo!(),
                ServerMessage::Sync(_) => todo!(),
                ServerMessage::Error(err) => {
                    error!(?err, "error received from server");
                }
            }
        }
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

/// Attempt to send a client message to the server.
async fn send_msg(tx: &mpsc::Sender<ClientUpdate>, message: ClientMessage) -> Result<()> {
    let update = ClientUpdate {
        client_message: Some(message),
    };
    tx.send(update)
        .await
        .context("failed to send message to server")
}
