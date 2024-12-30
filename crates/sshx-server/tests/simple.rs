use anyhow::Result;
use sshx::encrypt::Encrypt;
use sshx_core::proto::*;

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_rpc() -> Result<()> {
    let server = TestServer::new().await;
    let mut client = server.grpc_client().await;

    let req = OpenRequest {
        origin: "sshx.io".into(),
        encrypted_zeros: Encrypt::new("").zeros().into(),
        name: String::new(),
        write_password_hash: None,
    };
    let resp = client.open(req).await?;
    assert!(!resp.into_inner().name.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_web_get() -> Result<()> {
    let server = TestServer::new().await;

    let resp = reqwest::get(server.endpoint()).await?;
    assert!(!resp.status().is_server_error());

    Ok(())
}
