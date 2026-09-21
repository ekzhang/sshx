use anyhow::Result;
use sshx::encrypt::Encrypt;
use sshx_core::proto::*;
use sshx_server::web::protocol::WsClient;

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_list_files_rpc() -> Result<()> {
    let server = TestServer::new().await;
    let mut client = server.grpc_client().await;

    let req = OpenRequest {
        origin: "test".into(),
        encrypted_zeros: Encrypt::new("test").zeros().into(),
        name: "test-list".into(),
        write_password_hash: None,
    };
    let resp = client.open(req).await?;
    let session_name = resp.into_inner().name;

    let mut ws = ClientSocket::connect(
        &server.ws_endpoint(&session_name),
        "test",
        None,
    )
    .await?;
    ws.flush().await;
    assert!(ws.user_id.0 > 0, "should receive user ID");

    // Send ListFiles to verify message routing doesn't crash
    ws.send(WsClient::ListFiles("/".into())).await;
    ws.flush().await;
    assert!(ws.errors.is_empty(), "list files should not cause errors");

    Ok(())
}

#[tokio::test]
async fn test_file_operations_permission_check() -> Result<()> {
    let server = TestServer::new().await;
    let mut client = server.grpc_client().await;

    // Create a session with write password protection
    let req = OpenRequest {
        origin: "test".into(),
        encrypted_zeros: Encrypt::new("test").zeros().into(),
        name: "test-readonly".into(),
        write_password_hash: Some(Encrypt::new("writepass").zeros().into()),
    };
    let resp = client.open(req).await?;
    let session_name = resp.into_inner().name;

    // Connect without write password (read-only)
    let mut ws = ClientSocket::connect(
        &server.ws_endpoint(&session_name),
        "test",
        None,
    )
    .await?;
    ws.flush().await;
    assert!(ws.user_id.0 > 0, "should receive user ID");

    // DeleteFile should be rejected for read-only user
    ws.send(WsClient::DeleteFile("/test.txt".into())).await;
    ws.flush().await;
    assert!(
        !ws.errors.is_empty(),
        "should reject delete for read-only user"
    );

    Ok(())
}
