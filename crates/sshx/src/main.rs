use ansi_term::Color::{Cyan, Fixed, Yellow};
use anyhow::Result;
use clap::Parser;
use sshx::{controller::Controller, terminal::get_default_shell};
use tokio::signal;

/// Web-based, real-time collaboration for your remote terminal.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Address of the remote sshx server.
    #[clap(long, default_value = "https://sshx.io")]
    server: String,

    /// Local shell command to run in the terminal.
    #[clap(long)]
    shell: Option<String>,
}

fn print_greeting(shell: &str, controller: &Controller) {
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
│  {shell}   {shell_v:50}  │
│  {web_url} {url}{url_space}  │
│                                                               │
│  Your terminal is accessible on the web.                      │
│                                                               │
╰───────────────────────────────────────────────────────────────╯
"#,
        title = Cyan.paint(format!("sshx {version_str:54}")),
        shell = Fixed(8).paint("shell:"),
        shell_v = shell,
        web_url = Fixed(8).paint("web url:"),
        url = Yellow.underline().paint(controller.url()),
        url_space = " ".repeat(50 - controller.url().len()),
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let shell = args.shell.unwrap_or_else(get_default_shell);

    let mut controller = Controller::new(&args.server, &shell).await?;
    print_greeting(&shell, &controller);

    let exit_signal = signal::ctrl_c();
    tokio::pin!(exit_signal);
    tokio::select! {
        _ = controller.run() => unreachable!(),
        Ok(()) = &mut exit_signal => (),
    };
    controller.close().await?;

    Ok(())
}
