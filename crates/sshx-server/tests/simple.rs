use anyhow::Result;
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
