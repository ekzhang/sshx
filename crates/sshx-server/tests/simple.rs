use common::*;
use sshx_core::proto::*;

pub mod common;

#[tokio::test]
async fn test_rpc() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut client = greeter_client::GreeterClient::connect(server.endpoint())
        .await
        .unwrap();

    let req = HelloRequest {
        name: "adam".into(),
    };
    let resp = client.say_hello(req).await.unwrap();
    assert_eq!(&resp.into_inner().message, "Hello adam!");

    Ok(())
}
