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

/// A session name is interpolated into an upstream URL when proxying between
/// mesh peers, so the `{name}` path parameter must be restricted to characters
/// that cannot alter the meaning of that URL.
///
/// Each case performs a real WebSocket handshake, because that is the only
/// unambiguous discriminator: with validation in place the upgrade is refused
/// with `400`, whereas without it the upgrade succeeds and the handler runs.
/// A plain `GET` cannot distinguish the two, since `WebSocketUpgrade` also
/// rejects a non-upgrade request with `400`.
#[tokio::test]
async fn test_session_name_rejects_path_traversal() -> Result<()> {
    let server = TestServer::new().await;

    // Control: a legitimate name must still be allowed to connect.
    tokio_tungstenite::connect_async(server.ws_endpoint("aB3xY9zQ1mK")).await?;

    // These stay percent-encoded on the wire, since the `http` crate does not
    // normalize paths. The server decodes them into `/` and `.` before
    // validating, which is exactly what has to be rejected.
    let rejected = [
        "a%2Fb",              // decodes to "a/b"
        "a%2F..%2F..%2Fevil", // decodes to "a/../../evil"
        "%2E%2E%2Fadmin",     // decodes to "../admin"
        "%2E%2E",             // decodes to ".."
        "a%20b",              // decodes to "a b"
        "a+b",
    ];

    for name in rejected {
        let uri = format!("ws://{}/api/s/{}", server.local_addr(), name);
        let err = tokio_tungstenite::connect_async(&uri)
            .await
            .err()
            .unwrap_or_else(|| panic!("expected {name:?} to be refused"));

        match err {
            tokio_tungstenite::tungstenite::Error::Http(resp) => {
                assert_eq!(
                    resp.status(),
                    http::StatusCode::BAD_REQUEST,
                    "expected {name:?} to be refused with 400"
                );
            }
            other => panic!("expected an HTTP rejection for {name:?}, got {other}"),
        }
    }

    Ok(())
}
