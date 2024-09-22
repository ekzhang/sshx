#!/bin/sh

# This is a short script to install the latest version of the sshx binary.
#
# It's meant to be as simple as possible, so if you're not happy hardcoding a
# `curl | sh` pipe in your application, you can just download the binary
# directly with the appropriate URL for your architecture.
#
# If you'd like to run it without installing to /usr/local/bin, use `sh -s run`.
# To download to the current directory, use `sh -s download`.

set +e

case "$(uname -s)" in
  Linux*) suffix="-unknown-linux-musl";;
  Darwin*) suffix="-apple-darwin";;
  FreeBSD*) suffix="-unknown-freebsd";;
  MINGW*|MSYS*|CYGWIN*)
    echo "You are on Windows. Please visit sshx.io to download the executable.";
    exit 1;;
  *) echo "Unsupported OS $(uname -s)"; exit 1;;
esac

case "$(uname -m)" in
  aarch64 | aarch64_be | arm64 | armv8b | armv8l) arch="aarch64";;
  x86_64 | x64 | amd64) arch="x86_64";;
  armv6l) arch="arm"; suffix="${suffix}eabihf";;
  armv7l) arch="armv7"; suffix="${suffix}eabihf";;
  *) echo "Unsupported arch $(uname -m)"; exit 1;;
esac

url="https://s3.amazonaws.com/sshx/sshx-${arch}${suffix}.tar.gz"

if [ -z "$NO_COLOR" ]; then
  ansi_reset="\033[0m"
  ansi_info="\033[35;1m"
  ansi_error="\033[31m"
  ansi_underline="\033[4m"
fi

cmd=${1:-install}
temp=$(mktemp)

case $cmd in
  "run")
    path=$(mktemp -d)
    will_run=1
    ;;
  "download")
    path=$(pwd)
    ;;
  "install")
    path=/usr/local/bin
    ;;
  *)
    printf "${ansi_error}Error: Invalid command. Please use 'run', 'download', or 'install'.\n"
    exit 1
    ;;
esac

printf "${ansi_reset}${ansi_info}↯ Downloading sshx from ${ansi_underline}%s${ansi_reset}\n" "$url"
http_code=$(curl "$url" -o "$temp" -w "%{http_code}")
if [ "$http_code" -lt 200 ] || [ "$http_code" -gt 299 ]; then
  printf "${ansi_error}Error: Request had status code ${http_code}.\n"
  cat "$temp" 1>&2
  printf "${ansi_reset}\n"
  exit 1
fi

printf "\n${ansi_reset}${ansi_info}↯ Adding sshx binary to ${ansi_underline}%s${ansi_reset}\n" "$path"
if [ "$(id -u)" -ne 0 ] && [ "$path" = "/usr/local/bin" ]; then
  sudo tar xf "$temp" -C "$path" || exit 1
else
  tar xf "$temp" -C "$path" || exit 1
fi

printf "\n${ansi_reset}${ansi_info}↯ Done! You can now run sshx.${ansi_reset}\n"

if [ -n "$will_run" ]; then
  "$path/sshx"
  rm -f "$path/sshx"
fi
