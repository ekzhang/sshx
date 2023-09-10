#!/bin/bash

# Manually releases the latest binaries to AWS S3.
#
# This runs on my M1 Macbook Pro with cross-compilation toolchains. I think it's
# probably better to replace this script with a CI configuration later.

set +e

TARGET_CC=x86_64-unknown-linux-musl-cc \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-unknown-linux-musl-gcc \
cargo build --release --target x86_64-unknown-linux-musl

TARGET_CC=aarch64-unknown-linux-musl-cc \
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-unknown-linux-musl-gcc \
cargo build --release --target aarch64-unknown-linux-musl

SDKROOT=$(xcrun -sdk macosx13.3 --show-sdk-path) \
MACOSX_DEPLOYMENT_TARGET=$(xcrun -sdk macosx13.3 --show-sdk-platform-version) \
cargo build --release --target x86_64-apple-darwin

cargo build --release --target aarch64-apple-darwin

temp=$(mktemp)
targets=(
  x86_64-unknown-linux-musl
  aarch64-unknown-linux-musl
  x86_64-apple-darwin
  aarch64-apple-darwin
)
for target in "${targets[@]}"
do
  echo "compress: target/$target/release/sshx"
  tar czf $temp -C target/$target/release sshx
  aws s3 cp $temp s3://sshx/sshx-$target.tar.gz

  echo "compress: target/$target/release/sshx-server"
  tar czf $temp -C target/$target/release sshx-server
  aws s3 cp $temp s3://sshx/sshx-server-$target.tar.gz
done
