use ansi_term::Color::{Cyan, Yellow};
use ansi_term::Style;
use anyhow::Result;
use clap::Parser;
use sshx::controller::Controller;
use tokio::signal;

/// Web-based, real-time collaboration for your remote terminal.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Address of the remote sshx server.
    #[clap(short, long, default_value = "https://sshx.io")]
    server: String,
}

fn print_greeting(controller: &Controller) {
    let version_str = match option_env!("CARGO_PKG_VERSION") {
        Some(version) => format!("v{version}"),
        None => String::from("[dev]"),
    };

    println!(
        r#"
╭───────────────────────────────────────────────────────────────╮
│                                                               │
│  {title}  │
│                                                               │
│  {session} {name:50}  │
│  {web_url} {url}{url_space}  │
│                                                               │
│  Your terminal is accessible on the web.                      │
│                                                               │
╰───────────────────────────────────────────────────────────────╯
"#,
        title = Cyan.paint(format!("sshx {version_str:54}")),
        session = Style::new().dimmed().paint("session:"),
        name = controller.name(),
        web_url = Style::new().dimmed().paint("web url:"),
        url = Yellow.underline().paint(controller.url()),
        url_space = " ".repeat(50 - controller.url().len()),
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut controller = Controller::new(&args.server).await?;
    print_greeting(&controller);

    let exit_signal = signal::ctrl_c();
    tokio::pin!(exit_signal);
    tokio::select! {
        _ = controller.run() => unreachable!(),
        Ok(()) = &mut exit_signal => (),
    };
    controller.close().await?;

    Ok(())
}
