#!/usr/bin/env bash
set -e 

if [[ -z "$1" ]]
then
  echo "version mut be provided"
  exit 1;
fi

mkdir -p dist/musl
cargo build --release 
cp target/x86_64-unknown-linux-musl/release/adana dist/musl/adana-$1