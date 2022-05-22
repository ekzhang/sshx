use anyhow::Result;
use clap::Parser;
use sshx::get_default_shell;
use sshx_core::proto::{sshx_service_client::SshxServiceClient, CloseRequest, OpenRequest};
use tokio::signal;
use tracing::info;

/// Web-based, real-time collaboration for your remote terminal.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Address of the remote sshx server.
    #[clap(short, long, default_value = "https://sshx.io")]
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let shell = get_default_shell();
    info!(%shell, "using default shell");

    let args = Args::parse();
    info!(origin = %args.server, "connecting to server");

    let mut client = SshxServiceClient::connect(args.server.clone()).await?;

    let req = OpenRequest {
        origin: args.server.clone(),
    };
    let resp = client.open(req).await?.into_inner();
    let name = resp.name;
    info!(url = %resp.url, "opened new session");

    let exit_signal = signal::ctrl_c();
    tokio::pin!(exit_signal);

    (&mut exit_signal).await?;

    info!("closing session");
    let req = CloseRequest { name };
    client.close(req).await?;

    Ok(())
}
