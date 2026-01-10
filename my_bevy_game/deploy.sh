#!/bin/bash
set -e

echo "Building for Raspberry Pi..."
cargo build --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features

echo "Copying to Pi..."
scp target/aarch64-unknown-linux-gnu/release/my_bevy_game ayu@pi.local:~/

echo "Done! Binary deployed to ayu@pi.local:~/"