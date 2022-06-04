# sshx

Visit [sshx.io](https://sshx.io) to learn more about this tool.

## Development

Install [Rust 1.61+](https://www.rust-lang.org/),
[Node v16](https://nodejs.org/), [NPM v8](https://www.npmjs.com/), and
[mprocs](https://github.com/pvolok/mprocs). Then, run in your interactive
terminal

```shell
$ npm install
$ mprocs
```

This will compile and start the server, an instance of the client, and the web
frontend in parallel on your machine.

## Deployment

Application is containerized with Docker and deployed on
[Fly.io](https://fly.io/). To deploy, just run `flyctl deploy` as a user with
appropriate permissions.
