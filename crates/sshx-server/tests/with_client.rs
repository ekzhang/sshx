use anyhow::{Context, Result};
use sshx::controller::Controller;
use sshx_core::proto::{server_update::ServerMessage, TerminalData};
use tokio::time::{self, Duration};

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_handshake() -> Result<()> {
    let server = TestServer::new().await?;
    let controller = Controller::new(&server.endpoint(), "/bin/bash").await?;
    controller.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_command() -> Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    let server = TestServer::new().await?;
    let mut controller = Controller::new(&server.endpoint(), "/bin/bash").await?;

    let session = server
        .find_session(controller.name())
        .context("couldn't find session in server state")?;

    let updates = session.update_tx();
    updates.send(ServerMessage::CreateShell(1)).await?;

    let data = TerminalData {
        id: 1,
        data: "ls\r\n".into(),
        seq: 0,
    };
    updates.send(ServerMessage::Data(data)).await?;

    tokio::select! {
        _ = controller.run() => (),
        _ = time::sleep(Duration::from_millis(1000)) => (),
    };
    controller.close().await?;
    Ok(())
}
