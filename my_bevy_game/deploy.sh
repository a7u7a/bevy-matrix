#!/bin/bash
HOST=ayu@pi.local

set -e

echo "Building for Raspberry Pi..."
cargo build --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features

echo "Copying to Pi..."
scp target/aarch64-unknown-linux-gnu/release/my_bevy_game $HOST:~/

echo "Done! Binary deployed to $HOST:~/"