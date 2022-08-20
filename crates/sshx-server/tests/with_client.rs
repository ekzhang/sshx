use anyhow::{Context, Result};
use sshx::{controller::Controller, runner::Runner};
use sshx_core::proto::{server_update::ServerMessage, TerminalInput};
use tokio::time::{self, Duration};

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_handshake() -> Result<()> {
    let server = TestServer::new().await?;
    let controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    controller.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_command() -> Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    let server = TestServer::new().await?;
    let runner = Runner::Shell("/bin/bash".into());
    let mut controller = Controller::new(&server.endpoint(), runner).await?;

    let session = server
        .find_session(controller.name())
        .context("couldn't find session in server state")?;

    let updates = session.update_tx();
    updates.send(ServerMessage::CreateShell(1)).await?;

    let data = TerminalInput {
        id: 1,
        data: "ls\r\n".into(),
    };
    updates.send(ServerMessage::Input(data)).await?;

    tokio::select! {
        _ = controller.run() => (),
        _ = time::sleep(Duration::from_millis(1000)) => (),
    };
    controller.close().await?;
    Ok(())
}
