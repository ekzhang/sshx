use anyhow::Result;
use sshx::controller::Controller;

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_handshake() -> Result<()> {
    let server = TestServer::new().await?;
    let controller = Controller::new(&server.endpoint()).await?;
    controller.close().await?;
    Ok(())
}
