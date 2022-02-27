use anyhow::Result;
use sshx_core::proto::HelloRequest;

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_rpc() -> Result<()> {
    let server = TestServer::new()?;
    let mut client = server.grpc_client().await?;

    let req = HelloRequest {
        name: "adam".into(),
    };
    let resp = client.hello(req).await?;
    assert_eq!(&resp.into_inner().message, "Hello adam!");

    Ok(())
}
