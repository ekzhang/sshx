use anyhow::Result;
use reqwest::{redirect::Policy, Client};
use sshx_core::proto::*;

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_rpc() -> Result<()> {
    let server = TestServer::new()?;
    let mut client = server.grpc_client().await?;

    let req = OpenRequest {};
    let resp = client.open(req).await?;
    assert!(!resp.into_inner().name.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_web_get() -> Result<()> {
    let server = TestServer::new()?;

    let resp = reqwest::get(server.endpoint()).await?;
    assert!(!resp.status().is_server_error());

    Ok(())
}

#[tokio::test]
async fn test_web_tls_redirect() -> Result<()> {
    let server = TestServer::new()?;

    let client = Client::builder().redirect(Policy::none()).build()?;

    let resp = client
        .get(server.endpoint())
        .header("x-forwarded-proto", "http")
        .send()
        .await?;
    assert!(resp.status().is_redirection());

    let resp = client
        .get(server.endpoint())
        .header("x-forwarded-proto", "https")
        .send()
        .await?;
    assert!(!resp.status().is_redirection());

    Ok(())
}
