# Bevy + RGB Matrix Integration

## Current Status

This is a minimal Bevy app demonstrating platform-specific plugin configuration:

- **Mac**: Runs with window and rendering (`DefaultPlugins`)
- **Pi**: Runs headless with minimal plugins (`ScheduleRunnerPlugin` + `TimePlugin`)
- **Shared logic**: Basic ECS demo (entities, components, systems, timers)

## Roadmap

Milestones:

- [x] Implement base bevy project with cross-compiling. Runs in mac and in the pi
- [x] Add `rpi-rgb-led-matrix` library rust bindings, minimal working demo writing something to the screen using bevy in rust. Must run and compile in both targets but no output is expected in window mode.
- [x] Add basic render output. Create a minimal bevy setup that renders a cube in the a window (mac only)
- [x] Make the above scene compile on both targets
- [x] Add Bevy rendering to headless pi, minimal working demo that renders the scene from the previous milestone to the led matrix. Once access to the rendered bevy camera output buffer is confirmed we will try to write the frame to the matrix screen using the rpi-rgb-led-matrix rust bindings

## Pending

- Improve documentation
- Try a rendering approach that draws directly to the screen canvas instead of using double buffering swap stuff
- Add FPS diagnostics
- Improve video documenting: Remove screen flickering when taking videos of the screen for documentation purposes
- Add platform-specific controls demo (Basic camera orbit with mouse on mac, playstation controller on raspberry pi (details TBD))
- How would we switch to event-driven rendering on the Pi? (Instead of fixed timestep)
- Implement tooling to help speed up cross-compile development and testing
- Document GPU driver installation on the Pi
- Test dev profiles on the Pi

## Running on Mac

```bash
cargo run --features window
# or just:
cargo run
```

Opens a window and prints hello messages every 2 seconds.

**Test headless mode locally** (simulates Pi environment):

```bash
cargo run --no-default-features --features matrix
```

**Development profiles:**

```bash
# Standard dev build (optimized for iteration speed)
# - Your code: opt-level 1 (fast compile, decent runtime)
# - Dependencies: opt-level 3 (slow first compile, fast runtime)
cargo run

# Ultra-fast compilation for quick syntax checks
# - Everything: opt-level 0 (fastest compile, slowest runtime)
cargo build --profile dev-fast --features window

# Release build (maximum optimization)
cargo build --release
```

The dev profile balances compile time and runtime performance. First compile is slow (dependencies), but subsequent builds are fast since dependencies are cached.

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

### Cross-compile the game for Pi

On Mac:

```bash
./deploy.sh
```

Run on Pi:

```bash
sudo ./my_bevy_game
```

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

