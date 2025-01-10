FROM rust:alpine AS backend
WORKDIR /home/rust/src
RUN apk --no-cache add musl-dev openssl-dev protoc
RUN rustup component add rustfmt
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/rust/src/target \
    cargo build --release --bin sshx-server && \
    cp target/release/sshx-server /usr/local/bin

FROM node:lts-alpine AS frontend
RUN apk --no-cache add git
WORKDIR /usr/src/app
COPY . .
RUN npm ci
RUN npm run build

FROM alpine:latest
WORKDIR /root
COPY --from=frontend /usr/src/app/build build
COPY --from=backend /usr/local/bin/sshx-server .
CMD ["./sshx-server", "--listen", "::"]
