# sshx

A secure web-based, collaborative terminal.

**Features:**

- Run a single command to share your terminal with anyone.
- Resize, move windows, and freely zoom and pan on an infinite canvas.
- See other people's cursors moving in real time.
- Connect to the nearest server in a globally distributed mesh.
- End-to-end encryption with Argon2 and AES.
- Automatic reconnection and real-time latency estimates.
- Predictive echo for faster local editing (à la
  [Mosh](https://github.com/mobile-shell/mosh)).

Visit [sshx.io](https://sshx.io) to learn more.

## Installation

Just run this command to get the `sshx` binary for your platform.

```shell
curl -sSf https://sshx.io/get | sh
```

Supports Linux and MacOS, on both x86_64 and arm64 architectures. The
precompiled Linux binaries are statically linked.

## Development

Here's how to work on the project, if you want to contribute.

### Building from source

To build the latest version of the client from source, clone this repository and
run, with [Rust](https://rust-lang.com/) installed:

```shell
cargo install --path crates/sshx
```

This will compile the `sshx` binary and place it in your `~/.cargo/bin` folder.

### Workflow

Install [Rust 1.70+](https://www.rust-lang.org/),
[Node v18](https://nodejs.org/), [NPM v9](https://www.npmjs.com/), and
[mprocs](https://github.com/pvolok/mprocs). Then, run

```shell
$ npm install
$ mprocs
```

This will compile and start the server, an instance of the client, and the web
frontend in parallel on your machine.

## Deployment

The application servers are deployed on [Fly.io](https://fly.io/).

```shell
fly deploy
```
