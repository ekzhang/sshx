use std::{net::SocketAddr, process::ExitCode};

use anyhow::Result;
use clap::Parser;
use sshx_server::{Server, ServerOptions};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info};

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

    /// Secret used for signing session tokens.
    #[clap(long, env = "SSHX_SECRET")]
    secret: Option<String>,

    /// Override the origin URL returned by the Open() RPC.
    #[clap(long)]
    override_origin: Option<String>,
}

#[tokio::main]
async fn start(args: Args) -> Result<()> {
    let host = if args.host { "::" } else { "::1" };
    let addr = SocketAddr::new(host.parse()?, args.port);

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    let mut options = ServerOptions::default();
    options.secret = args.secret;
    options.override_origin = args.override_origin;

    let server = Server::new(options);

    let serve_task = async {
        info!("server listening at {addr}");
        server.bind(&addr).await
    };

    let signals_task = async {
        tokio::select! {
            Some(()) = sigterm.recv() => (),
            Some(()) = sigint.recv() => (),
            else => return Ok(()),
        }
        info!("gracefully shutting down...");
        server.shutdown();
        Ok(())
    };

    tokio::try_join!(serve_task, signals_task)?;
    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or("info".into()))
        .with_writer(std::io::stderr)
        .init();

    match start(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err:?}");
            ExitCode::FAILURE
        }
    }
}
