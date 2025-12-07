# Bevy + RGB Matrix integration

(WIP)

## Deploying on Mac

Run on Mac:

```bash
cargo run --features window
```

This creates a 64x64 window with a test pattern (alternating red/blue checkerboard).

## Deploying to Raspberry Pi

### Cross compiling setup

**On the Mac:**

1. Add the ARM64 Linux target:

```bash
rustup target add aarch64-unknown-linux-gnu
```

Install a cross-linker (via Homebrew):

```bash
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu
```

2. Cross-compile the matrix library:

```bash
git clone https://github.com/hzeller/rpi-rgb-led-matrix.git
cd rpi-rgb-led-matrix
```

Then, build using the cross-compiler:

```bash
make -C lib \
  CC=aarch64-unknown-linux-gnu-gcc \
  CXX=aarch64-unknown-linux-gnu-g++ \
  AR=aarch64-unknown-linux-gnu-ar
```

Create a directory for cross-compile libraries and copy:

```bash
mkdir -p ~/cross-libs/aarch64-linux-gnu
cp lib/librgbmatrix.a ~/cross-libs/aarch64-linux-gnu/
```

Now we have a basic cross compiling setup for the game.

### Cross compile the game for Pi

Build the game for Raspberry Pi from Mac:

```bash
cargo build --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features
```

Copy the binary to Pi:

```bash
scp target/aarch64-unknown-linux-gnu/release/my_bevy_game ayu@pi.local:~/
```

4. Run on Pi (Must run as root for GPIO access)

```bash
sudo ./my_bevy_game
```

## Performance Tips for Pi

1. **Reduce color depth**: Use `--led-pwm-bits=7` (configure in matrix backend)
2. **Reserve CPU core**: Add `isolcpus=3` to `/boot/cmdline.txt`
3. **Target 30 FPS**: Realistic for Pi Zero 2 W with 64x64 display
4. **Disable swap**: Reduces latency

## Troubleshooting

### Error: "Pi sound module is loaded"

```bash
# Edit the config file
sudo nano /boot/firmware/config.txt

# Add this line at the end:
dtparam=audio=off

# Also blacklist the module
echo "blacklist snd_bcm2835" | sudo tee /etc/modprobe.d/blacklist-rgb-matrix.conf

# Reboot
sudo reboot
```

### Matrix shows garbage

- Try different `--led-slowdown-gpio` values (1, 2, 3)
- Check panel wiring and power supply

### Low frame rate

- Reduce `--led-pwm-bits`
- Use `--led-pwm-dither-bits=1`
- Simplify game rendering
