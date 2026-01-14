# Bevy + RGB Matrix test

Minimal test to confirm RGB Matrix + Bevy functionality

```bash
cargo build --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features
```

```bash
scp target/aarch64-unknown-linux-gnu/release/bevy_screen_demo ayu@pi.local~/
```