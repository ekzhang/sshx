use std::env;
use std::fs::File as StdFile;
use std::os::unix::io::{FromRawFd, RawFd};
use std::process::Command;
use std::time::Duration;

use nix::{pty, unistd::dup};
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time;

/// Returns the default shell on this system.
fn get_default_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"))
}

fn child_task(shell: String, tty: RawFd) -> anyhow::Result<()> {
    let tty2 = dup(tty)?;
    let tty3 = dup(tty)?;
    let stdin = unsafe { StdFile::from_raw_fd(tty) };
    let stdout = unsafe { StdFile::from_raw_fd(tty2) };
    let stderr = unsafe { StdFile::from_raw_fd(tty3) };
    let mut child = Command::new(&shell)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()?;
    child.wait()?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let shell = get_default_shell();
    println!("Using default shell: {shell}");

    let ports = pty::openpty(None, None)?;
    std::thread::spawn(move || {
        child_task(shell, ports.slave).expect("Child failed");
    });

    let master = unsafe { File::from_raw_fd(ports.master) };
    let (mut master_read, mut master_write) = io::split(master);
    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();

    tokio::try_join!(
        async {
            // time::sleep(Duration::from_secs(1)).await;
            master_write.write_all(b"ls\n").await
        },
        // async {
        //     let mut buf = String::new();
        //     stdin.read_line(&mut buf).await?;
        //     println!("Okay! {:?} -> {buf}", buf.as_bytes());
        //     master_write.write_all(buf.as_bytes()).await?;
        //     Ok::<_, io::Error>(())
        // },
        async {
            let mut read = BufReader::new(master_read);
            for _ in 0..5 {
                let mut buf = String::new();
                read.read_line(&mut buf).await?;
                print!("{}", buf);
            }
            Ok(())
        },
        // io::copy(&mut stdin, &mut master_write),
        // io::copy(&mut master_read, &mut stdout),
    )?;

    Ok(())

    // loop {
    //     let input_future = stdin.read(&mut buf);
    //     let output_future = master.read(&mut buf2);
    //     tokio::select! {
    //         num_read = input_future => {
    //             master.write_all(&buf[0..num_read?]).await?;
    //         }
    //         num_read = output_future => {
    //             stdout.write_all(&buf2[0..num_read?]).await?;
    //             stdout.flush().await?;
    //         }
    //     };
    // }
}
