FROM rust:alpine as backend
WORKDIR /home/rust/src
RUN apk --no-cache add musl-dev openssl-dev protoc
RUN rustup component add rustfmt
COPY . .
RUN cargo build --release --bin sshx-server

FROM node:lts-alpine as frontend
RUN apk --no-cache add git
WORKDIR /usr/src/app
COPY . .
RUN npm ci
RUN npm run build

FROM alpine:latest
WORKDIR /root
COPY --from=frontend /usr/src/app/build build
COPY --from=backend /home/rust/src/target/release/sshx-server .
CMD ["./sshx-server", "--host"]
