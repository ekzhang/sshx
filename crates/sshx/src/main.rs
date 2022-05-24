use anyhow::Result;
use clap::Parser;
use sshx::{controller::Controller, terminal::get_default_shell};
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

    let controller = Controller::new(&args.server).await?;

    let exit_signal = signal::ctrl_c();
    tokio::pin!(exit_signal);

    (&mut exit_signal).await?;
    controller.close().await?;

    Ok(())
}
