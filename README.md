# sshx

**The delightful web-based, collaborative terminal you never knew you needed.**

Visit [sshx.io](https://sshx.io) to learn more, including installation steps.

## Development

### Building from source

To build the latest version of the client from source, clone this repository and
run, with [Rust](https://rust-lang.com/) installed:

```
cargo install --path crates/sshx
```

This will compile the `sshx` binary and place it in your `~/.cargo/bin` folder.

### Workflow

Install [Rust 1.66+](https://www.rust-lang.org/),
[Node v16](https://nodejs.org/), [NPM v8](https://www.npmjs.com/), and
[mprocs](https://github.com/pvolok/mprocs). Then, run

```shell
$ npm install
$ mprocs
```

This will compile and start the server, an instance of the client, and the web
frontend in parallel on your machine.

## Deployment

This application is containerized and deployed on [Fly.io](https://fly.io/). To
set up a new server, just run `flyctl deploy` as a user with the appropriate
permissions.
