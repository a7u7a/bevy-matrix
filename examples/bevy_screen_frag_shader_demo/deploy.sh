#!/bin/bash
HOST=ayu@pi.local
CRATE="bevy_screen_frag_shader_demo"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Building for Raspberry Pi..."
cd "$WORKSPACE_ROOT"
cargo build -p "$CRATE" --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features

BINARY_PATH="$WORKSPACE_ROOT/target/aarch64-unknown-linux-gnu/release/$CRATE"

SIZE_MB=$(du -m "$BINARY_PATH" | cut -f1)
echo "Binary size: ${SIZE_MB} MB"

echo "Copying to Pi..."
scp "$BINARY_PATH" "$HOST:~/"

echo "Done! Binary deployed to $HOST:~/"
