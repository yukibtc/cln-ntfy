#!/bin/bash
set -x
script_dir=$(dirname -- "$(readlink -f -- "$0")")
cargo_toml_path="$script_dir/../Cargo.toml"

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
cargo build --manifest-path="$cargo_toml_path"
