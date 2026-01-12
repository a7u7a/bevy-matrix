#!/bin/bash
HOST=ayu@pi.local

set -e

echo "Building for Raspberry Pi..."
cargo build --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features

echo "Copying to Pi..."
scp target/aarch64-unknown-linux-gnu/release/bevy_screen_gpu_demo $HOST:~/

echo "Done! Binary deployed to $HOST:~/"

