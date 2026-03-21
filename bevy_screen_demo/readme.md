# Bevy + RGB Matrix test

RGB Matrix + Bevy smoke test

```bash
cargo build --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features
```

```bash
scp target/aarch64-unknown-linux-gnu/release/bevy_screen_demo ayu@pi.local~/
```