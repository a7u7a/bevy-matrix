#!/bin/bash
HOST=ayu@pi.local

set -e

echo "Building for Raspberry Pi..."
cargo build --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features

BINARY_PATH=target/aarch64-unknown-linux-gnu/release/bevy_screen_frag_shader_demo

# Get binary size in MB (rounded)
SIZE_MB=$(du -m "$BINARY_PATH" | cut -f1)

echo "Binary size: ${SIZE_MB} MB"

echo "Copying to Pi..."
scp $BINARY_PATH $HOST:~/

echo "Done! Binary and assets deployed to $HOST:~/"
