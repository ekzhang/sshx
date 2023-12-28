# sshx

A secure web-based, collaborative terminal.

![](https://i.imgur.com/Q3qKAHW.png)

**Features:**

- Run a single command to share your terminal with anyone.
- Resize, move windows, and freely zoom and pan on an infinite canvas.
- See other people's cursors moving in real time.
- Connect to the nearest server in a globally distributed mesh.
- End-to-end encryption with Argon2 and AES.
- Automatic reconnection and real-time latency estimates.
- Predictive echo for faster local editing (à la Mosh).

Visit [sshx.io](https://sshx.io) to learn more.

## Installation

Just run this command to get the `sshx` binary for your platform.

```shell
curl -sSf https://sshx.io/get | sh
```

Supports Linux and MacOS, on both x86_64 and arm64 architectures. The
precompiled Linux binaries are statically linked.

### CI/CD

You can also use sshx in continuous integration workflows to help debug tricky
issues, like in GitHub Actions.

```yaml
name: CI
on: push

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # ... other steps ...

      - run: curl -sSf https://sshx.io/get | sh && sshx
      #      ^
      #      └ This will open a remote terminal session and print the URL. It
      #        should take under a second.
```

We don't have a prepackaged action because it's just a single command. It works
anywhere: GitLab CI, CircleCI, Buildkite, CI on your Raspberry Pi, etc.

Be careful adding this to a public GitHub repository, as any user can view the
logs of a CI job while it is running.

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

First, start service containers for development.

```shell
docker compose up -d
```

Install [Rust 1.70+](https://www.rust-lang.org/),
[Node v18](https://nodejs.org/), [NPM v9](https://www.npmjs.com/), and
[mprocs](https://github.com/pvolok/mprocs). Then, run

```shell
npm install
mprocs
```

This will compile and start the server, an instance of the client, and the web
frontend in parallel on your machine.

## Deployment

I host the application servers on [Fly.io](https://fly.io/) and with
[Redis Cloud](https://redis.com/).

Self-hosted deployments are not supported at the moment. If you want to deploy
sshx, you'll need to properly implement HTTP/TCP reverse proxies, gRPC
forwarding, TLS termination, private mesh networking, and graceful shutdown.

Please do not run the development commands in a public setting, as this is
insecure.
