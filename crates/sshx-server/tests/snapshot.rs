use std::sync::Arc;

use anyhow::Result;
use sshx::{controller::Controller, runner::Runner};
use sshx_core::{Sid, Uid};
use sshx_server::{
    session::Session,
    web::protocol::{WsClient, WsWinsize},
};

use crate::common::*;

pub mod common;

#[tokio::test]
async fn test_basic_restore() -> Result<()> {
    let server = TestServer::new().await;

    let mut controller = Controller::new(&server.endpoint(), "", Runner::Echo, false).await?;
    let name = controller.name().to_owned();
    let key = controller.encryption_key().to_owned();
    tokio::spawn(async move { controller.run().await });

    let mut s = ClientSocket::connect(&server.ws_endpoint(&name), &key, None).await?;
    s.flush().await;
    assert_eq!(s.user_id, Uid(1));

    s.send(WsClient::Create(0, 0)).await;
    s.flush().await;

    let new_size = WsWinsize {
        x: 42,
        y: 105,
        rows: 200,
        cols: 20,
    };

    s.send_input(Sid(1), b"hello there!").await;
    s.send_input(Sid(1), b" - another message").await;
    s.send(WsClient::Move(Sid(1), Some(new_size))).await;
    s.flush().await;
    assert!(s.shells.contains_key(&Sid(1)));

    // Replace the shell with its snapshot.
    let data = server.state().lookup(&name).unwrap().snapshot()?;
    server
        .state()
        .insert(&name, Arc::new(Session::restore(&data)?));

    let mut s = ClientSocket::connect(&server.ws_endpoint(&name), &key, None).await?;
    s.send(WsClient::Subscribe(Sid(1), 0)).await;
    s.flush().await;

    assert_eq!(s.read(Sid(1)), "hello there! - another message");
    assert_eq!(s.shells.get(&Sid(1)).unwrap(), &new_size);

    Ok(())
}
