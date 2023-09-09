use anyhow::{Context, Result};
use sshx::{controller::Controller, encrypt::Encrypt, runner::Runner};
use sshx_core::{
    proto::{server_update::ServerMessage, TerminalInput},
    Sid, Uid,
};
use sshx_server::web::protocol::{WsClient, WsWinsize};
use tokio::time::{self, Duration};

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_handshake() -> Result<()> {
    let server = TestServer::new().await;
    let controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    controller.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_command() -> Result<()> {
    let server = TestServer::new().await;
    let runner = Runner::Shell("/bin/bash".into());
    let mut controller = Controller::new(&server.endpoint(), runner).await?;

    let session = server
        .find_session(controller.name())
        .context("couldn't find session in server state")?;

    let updates = session.update_tx();
    updates.send(ServerMessage::CreateShell(1)).await?;

    let key = controller.encryption_key();
    let encrypt = Encrypt::new(key);
    let offset = 4242;
    let data = TerminalInput {
        id: 1,
        data: encrypt.segment(0x200000000, offset, b"ls\r\n"),
        offset,
    };
    updates.send(ServerMessage::Input(data)).await?;

    tokio::select! {
        _ = controller.run() => (),
        _ = time::sleep(Duration::from_millis(1000)) => (),
    };
    controller.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_ws_missing() -> Result<()> {
    let server = TestServer::new().await;

    let bad_endpoint = format!("ws://{}/not/an/endpoint", server.local_addr());
    assert!(ClientSocket::connect(&bad_endpoint, "").await.is_err());

    let mut s = ClientSocket::connect(&server.ws_endpoint("foobar"), "").await?;
    s.expect_close(4404).await;

    Ok(())
}

#[tokio::test]
async fn test_ws_basic() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let mut s = ClientSocket::connect(&server.ws_endpoint(&name), &key).await?;
    s.flush().await;
    assert_eq!(s.user_id, Uid(1));

    s.send(WsClient::Create()).await;
    s.flush().await;
    assert_eq!(s.shells.len(), 1);
    assert!(s.shells.contains_key(&Sid(1)));

    s.send(WsClient::Subscribe(Sid(1), 0)).await;
    assert_eq!(s.read(Sid(1)), "");

    s.send_input(Sid(1), b"hello!").await;
    s.flush().await;
    assert_eq!(s.read(Sid(1)), "hello!");

    s.send_input(Sid(1), b" 123").await;
    s.flush().await;
    assert_eq!(s.read(Sid(1)), "hello! 123");

    Ok(())
}

#[tokio::test]
async fn test_ws_resize() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let mut s = ClientSocket::connect(&server.ws_endpoint(&name), &key).await?;

    s.send(WsClient::Move(Sid(1), None)).await; // error: does not exist yet!
    s.flush().await;
    assert_eq!(s.errors.len(), 1);

    s.send(WsClient::Create()).await;
    s.flush().await;
    assert_eq!(s.shells.len(), 1);
    assert_eq!(*s.shells.get(&Sid(1)).unwrap(), WsWinsize::default());

    let new_size = WsWinsize {
        x: 42,
        y: 105,
        rows: 200,
        cols: 20,
    };
    s.send(WsClient::Move(Sid(1), Some(new_size))).await;
    s.send(WsClient::Move(Sid(2), Some(new_size))).await; // error: does not exist
    s.flush().await;
    assert_eq!(s.shells.len(), 1);
    assert_eq!(*s.shells.get(&Sid(1)).unwrap(), new_size);
    assert_eq!(s.errors.len(), 2);

    s.send(WsClient::Close(Sid(1))).await;
    s.flush().await;
    assert_eq!(s.shells.len(), 0);

    s.send(WsClient::Move(Sid(1), None)).await; // error: shell was closed
    s.flush().await;
    assert_eq!(s.errors.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_users_join() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let endpoint = server.ws_endpoint(&name);
    let mut s1 = ClientSocket::connect(&endpoint, &key).await?;
    s1.flush().await;
    assert_eq!(s1.users.len(), 1);

    let mut s2 = ClientSocket::connect(&endpoint, &key).await?;
    s2.flush().await;
    assert_eq!(s2.users.len(), 2);

    drop(s2);
    let mut s3 = ClientSocket::connect(&endpoint, &key).await?;
    s3.flush().await;
    assert_eq!(s3.users.len(), 2);

    s1.flush().await;
    assert_eq!(s1.users.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_users_metadata() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let endpoint = server.ws_endpoint(&name);
    let mut s = ClientSocket::connect(&endpoint, &key).await?;
    s.flush().await;
    assert_eq!(s.users.len(), 1);
    assert_eq!(s.users.get(&s.user_id).unwrap().cursor, None);

    s.send(WsClient::SetName("mr. foo".into())).await;
    s.send(WsClient::SetCursor(Some((40, 524)))).await;
    s.flush().await;
    let user = s.users.get(&s.user_id).unwrap();
    assert_eq!(user.name, "mr. foo");
    assert_eq!(user.cursor, Some((40, 524)));

    Ok(())
}

#[tokio::test]
async fn test_chat_messages() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let endpoint = server.ws_endpoint(&name);
    let mut s1 = ClientSocket::connect(&endpoint, &key).await?;
    let mut s2 = ClientSocket::connect(&endpoint, &key).await?;

    s1.send(WsClient::SetName("billy".into())).await;
    s1.send(WsClient::Chat("hello there!".into())).await;
    s1.flush().await;

    s2.flush().await;
    assert_eq!(s2.messages.len(), 1);
    assert_eq!(
        s2.messages[0],
        (s1.user_id, "billy".into(), "hello there!".into())
    );

    let mut s3 = ClientSocket::connect(&endpoint, &key).await?;
    s3.flush().await;
    assert_eq!(s1.messages.len(), 1);
    assert_eq!(s3.messages.len(), 0);

    Ok(())
}
