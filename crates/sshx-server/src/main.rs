use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use sshx_server::make_server_bind;
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

/// The sshx server CLI interface.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Specify port to listen on.
    #[clap(long, default_value_t = 8051)]
    port: u16,

    /// Whether to expose the server on all network interfaces.
    #[clap(long)]
    host: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let host = if args.host { "::" } else { "::1" };
    let addr = SocketAddr::new(host.parse()?, args.port);

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    info!("server listening at {addr}");
    make_server_bind(&addr, async {
        tokio::select! {
            _ = sigterm.recv() => (),
            _ = sigint.recv() => (),
        }
        info!("gracefully shutting down...");
    })
    .await?;
    Ok(())
}
