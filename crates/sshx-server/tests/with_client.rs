use anyhow::{Context, Result};
use sshx::{controller::Controller, runner::Runner};
use sshx_core::{
    proto::{server_update::ServerMessage, TerminalInput},
    Sid, Uid,
};
use sshx_server::web::{WsClient, WsWinsize};
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

#[tokio::test]
async fn test_ws_missing() -> Result<()> {
    let server = TestServer::new().await?;

    let bad_endpoint = format!("ws://{}/not/an/endpoint", server.local_addr());
    assert!(ClientSocket::connect(&bad_endpoint).await.is_err());

    let mut stream = ClientSocket::connect(&server.ws_endpoint("foobar")).await?;
    stream.expect_close(4404).await;

    Ok(())
}

#[tokio::test]
async fn test_ws_basic() -> Result<()> {
    let server = TestServer::new().await?;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    tokio::spawn(async move { controller.run().await });

    let mut stream = ClientSocket::connect(&server.ws_endpoint(&name)).await?;
    stream.flush().await;
    assert_eq!(stream.user_id, Uid(1));

    stream.send(WsClient::Create()).await;
    stream.flush().await;
    assert_eq!(stream.shells.len(), 1);
    assert!(stream.shells.contains_key(&Sid(1)));

    stream.send(WsClient::Subscribe(Sid(1), 0)).await;
    assert_eq!(stream.read(Sid(1)), "");

    stream
        .send(WsClient::Data(Sid(1), b"hello!".to_vec()))
        .await;
    stream.flush().await;
    assert_eq!(stream.read(Sid(1)), "hello!");

    stream.send(WsClient::Data(Sid(1), b" 123".to_vec())).await;
    stream.flush().await;
    assert_eq!(stream.read(Sid(1)), "hello! 123");

    Ok(())
}

#[tokio::test]
async fn test_ws_resize() -> Result<()> {
    let server = TestServer::new().await?;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    tokio::spawn(async move { controller.run().await });

    let mut stream = ClientSocket::connect(&server.ws_endpoint(&name)).await?;

    stream.send(WsClient::Move(Sid(1), None)).await; // error: does not exist yet!
    stream.flush().await;
    assert_eq!(stream.errors.len(), 1);

    stream.send(WsClient::Create()).await;
    stream.flush().await;
    assert_eq!(stream.shells.len(), 1);
    assert_eq!(*stream.shells.get(&Sid(1)).unwrap(), WsWinsize::default());

    let new_size = WsWinsize {
        x: 42,
        y: 105,
        rows: 200,
        cols: 20,
    };
    stream.send(WsClient::Move(Sid(1), Some(new_size))).await;
    stream.send(WsClient::Move(Sid(2), Some(new_size))).await; // error: does not exist
    stream.flush().await;
    assert_eq!(stream.shells.len(), 1);
    assert_eq!(*stream.shells.get(&Sid(1)).unwrap(), new_size);
    assert_eq!(stream.errors.len(), 2);

    stream.send(WsClient::Close(Sid(1))).await;
    stream.flush().await;
    assert_eq!(stream.shells.len(), 0);

    stream.send(WsClient::Move(Sid(1), None)).await; // error: shell was closed
    stream.flush().await;
    assert_eq!(stream.errors.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_users_join() -> Result<()> {
    let server = TestServer::new().await?;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    tokio::spawn(async move { controller.run().await });

    let endpoint = server.ws_endpoint(&name);
    let mut stream1 = ClientSocket::connect(&endpoint).await?;
    stream1.flush().await;
    assert_eq!(stream1.users.len(), 1);

    let mut stream2 = ClientSocket::connect(&endpoint).await?;
    stream2.flush().await;
    assert_eq!(stream2.users.len(), 2);

    drop(stream2);
    let mut stream3 = ClientSocket::connect(&endpoint).await?;
    stream3.flush().await;
    assert_eq!(stream3.users.len(), 2);

    stream1.flush().await;
    assert_eq!(stream1.users.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_users_metadata() -> Result<()> {
    let server = TestServer::new().await?;

    let mut controller = Controller::new(&server.endpoint(), Runner::Echo).await?;
    let name = controller.name().to_owned();
    tokio::spawn(async move { controller.run().await });

    let endpoint = server.ws_endpoint(&name);
    let mut stream = ClientSocket::connect(&endpoint).await?;
    stream.flush().await;
    assert_eq!(stream.users.len(), 1);
    assert_eq!(stream.users.get(&stream.user_id).unwrap().cursor, None);

    stream.send(WsClient::SetName("mr. foo".into())).await;
    stream.send(WsClient::SetCursor(Some((40, 524)))).await;
    stream.flush().await;
    let user = stream.users.get(&stream.user_id).unwrap();
    assert_eq!(user.name, "mr. foo");
    assert_eq!(user.cursor, Some((40, 524)));

    Ok(())
}
