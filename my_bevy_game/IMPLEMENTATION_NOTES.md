# Bevy RGB Matrix Implementation - Complete!

## What Was Built

A portable Bevy game that can run on both Mac (for development) and Raspberry Pi (outputting to RGB LED matrix).

## Project Structure

```
my_bevy_game/
├── Cargo.toml                  # Features: window (Mac) and matrix (Pi)
├── src/
│   ├── main.rs                # Main app with conditional plugin setup
│   ├── display/
│   │   ├── mod.rs            # DisplayBackend trait + Resource wrapper
│   │   ├── window.rs         # Mac development backend
│   │   └── matrix.rs         # Pi LED matrix backend
```

## Testing on Mac

The code builds and works on Mac:

```bash
cd /Users/userfriendly/code/rpi-wgpu/my_bevy_game
cargo run --features window
```

This creates a 64x64 window with a test pattern (alternating red/blue checkerboard).

## Deploying to Raspberry Pi

### Step 1: Install Dependencies on Pi

SSH into your Raspberry Pi and run:

```bash
# Install the rpi-rgb-led-matrix C++ library
cd /tmp
git clone https://github.com/hzeller/rpi-rgb-led-matrix.git
cd rpi-rgb-led-matrix
make -C lib
sudo make -C lib install

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
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

## Files Modified

- `Cargo.toml` - Added features and conditional dependencies
- `src/main.rs` - Conditional plugin setup
- `src/display/mod.rs` - Created DisplayBackend trait
- `src/display/window.rs` - Window backend for Mac
- `src/display/matrix.rs` - Matrix backend for Pi

All original game logic (Person/Name components) preserved!

