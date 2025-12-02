# Bevy + RGB Matrix integration

(WIP)

## Deploying on Mac

Run on Mac:

```bash
cargo run --features window
```

This creates a 64x64 window with a test pattern (alternating red/blue checkerboard).

## Deploying to Raspberry Pi

### Cross compile

On your Mac:

Add the ARM64 Linux target

```bash
rustup target add aarch64-unknown-linux-gnu
```

Install a cross-linker (via Homebrew):

```bash
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu
```

Cross-compile the matrix library

```bash
git clone https://github.com/hzeller/rpi-rgb-led-matrix.git
cd rpi-rgb-led-matrix
```

Then, build with the cross-compiler:

```bash
make -C lib \
  CC=aarch64-unknown-linux-gnu-gcc \
  CXX=aarch64-unknown-linux-gnu-g++ \
  AR=aarch64-unknown-linux-gnu-ar
```

Create a directory for cross-compile libraries:

```bash
mkdir -p ~/cross-libs/aarch64-linux-gnu
```

Copy to the cross-libs location:

```bash
cp lib/librgbmatrix.a ~/cross-libs/aarch64-linux-gnu/
```

Build for Pi from Mac:

```bash
cargo build --release --target aarch64-unknown-linux-gnu --features matrix --no-default-features
```

Copy the binary to Pi:

```bash
scp target/aarch64-unknown-linux-gnu/release/my_bevy_game ayu@pi.local:~/
```

Run on Pi (Must run as root for GPIO access)

```bash
sudo ./my_bevy_game
```

#### Troubleshooting " Pi sound module is loaded" error

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

### Step 1: Install Dependencies on Pi

#### Install the `rpi-rgb-led-matrix` library in the pi

SSH into your Raspberry Pi and run:

```bash
# Install the rpi-rgb-led-matrix C++ library
cd ~
git clone https://github.com/hzeller/rpi-rgb-led-matrix.git
cd rpi-rgb-led-matrix
make  # Build from root directory (not lib/)

# The library files are now in ~/rpi-rgb-led-matrix/lib/
# No system-wide install needed - we link directly to this path
```

#### Install Rust in Raspberry Pi Zero 2 W

(This failed because there is not enough ram (512mb) on the zero to successfully run the rust install script)

Increase swap space:

```bash
# Create a 4GB swap file
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Verify swap is active
free -h
```

Optional: To make the swap permanent (survives reboot), add this line to `/etc/fstab`:

```
/swapfile none swap sw 0 0
```

Install Rust with minimal profile:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal
```

Reload terminal:

```bash
source $HOME/.cargo/env
```

In case the install fails:

```bash
rustup self uninstall
```

### Installing with `screen`

In case the installer takes too long and we get disconnected from the pi:

```bash
# Install screen (if not already installed)
sudo apt-get install screen -y

# Start a screen session
screen -S rust_install

# Clean up any partial installation
rm -rf ~/.rustup ~/.cargo

# Run the installer again, now inside screen
```

If you get disconnected:

```bash
# SSH back in and reattach to the session
screen -r rust_install
```

#### Alternative: Keep SSH alive

Add this to your local Mac's `~/.ssh/config`:

```
ServerAliveInterval 60
ServerAliveCountMax 3
```

Or run with the option directly:

```bash
ssh -o ServerAliveInterval=60 ayu@pi.local
```

### Step 2: Build on Pi

Copy your project to the Pi and build there:

```bash
# On your Mac
scp -r /Users/userfriendly/code/rpi-wgpu/my_bevy_game pi@raspberrypi.local:~/

# SSH to Pi
ssh pi@raspberrypi.local

# Build on Pi
cd ~/my_bevy_game
cargo build --release --features matrix --no-default-features
```

### Step 3: Run on Pi

```bash
# Must run as root for GPIO access
sudo ./target/release/my_bevy_game
```

## How It Works

### Architecture

1. **DisplayBackend trait**: Abstracts the display interface
2. **Feature flags**:
   - `window` - Enables Bevy's windowing system (Mac)
   - `matrix` - Enables rpi-led-matrix bindings (Pi)
3. **DisplayResource**: Bevy Resource wrapper with unsafe Send/Sync impl (safe because Bevy ensures single-threaded access)

### Code Flow

1. Initialize backend based on feature flag
2. Wrap in DisplayResource and add to Bevy app
3. `render_frame` system generates pixel data (RGB24 format)
4. Backend writes pixels to window or matrix

### Current Demo

The `render_frame` function creates a simple alternating red/blue checkerboard pattern as a test. You can replace this with actual game rendering.

## Next Steps for Game Development

1. **Replace test pattern**: Modify `render_frame()` to capture actual Bevy camera output
2. **Add game logic**: Your existing Person/Name ECS components are preserved
3. **Optimize performance**: Adjust frame rate and rendering quality for Pi Zero 2 W

## Performance Tips for Pi

1. **Reduce color depth**: Use `--led-pwm-bits=7` (configure in matrix backend)
2. **Reserve CPU core**: Add `isolcpus=3` to `/boot/cmdline.txt`
3. **Target 30 FPS**: Realistic for Pi Zero 2 W with 64x64 display
4. **Disable swap**: Reduces latency

## Troubleshooting

### "Operation not permitted" on Pi

- Must run with `sudo` for GPIO access

### Matrix shows garbage

- Try different `--led-slowdown-gpio` values (1, 2, 3)
- Check panel wiring and power supply

### Low frame rate

- Reduce `--led-pwm-bits`
- Use `--led-pwm-dither-bits=1`
- Simplify game rendering
