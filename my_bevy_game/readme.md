# Bevy + RGB Matrix Integration

(WIP)

## Current Status

This is a minimal Bevy app demonstrating platform-specific plugin configuration:

- **Mac**: Runs with window and rendering (`DefaultPlugins`)
- **Pi**: Runs headless with minimal plugins (`ScheduleRunnerPlugin` + `TimePlugin`)
- **Shared logic**: Basic ECS demo (entities, components, systems, timers)

## Roadmap

- [x] Implement base bevy project with cross-compiling. Runs in mac and in the pi
- [ ] (In progress) Add `rpi-rgb-led-matrix` library rust bindings, minimal working demo writing something to the screen using bevy in rust. Must run and compile in both targets
- [ ] Add Bevy rendering in headless rpi, minimal working demo. Can we render in a context that has no window? can we access the camera or some way of rendering target?
- [ ] Integrate bevy and matrix: Once access to the rendered bevy camera output buffer is confirmed we will try to write the frame to the matrix screen using the rpi-rgb-led-matrix rust bindings

## Pending

- Implement tooling to help speed up cross-compile development and testing
- Document rpi gpu driver installation

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

Run on Pi:

```bash
./my_bevy_game
```

**Expected output:**

```
hello Elaina Hume!
hello Renzo Hume!
hello Zayna Nieves!
...
```

Repeating every 2 seconds. (Use `sudo ./my_bevy_game` when matrix control is added for GPIO access.)

## How It Works

The code uses conditional compilation to select appropriate plugins:

```rust
#[cfg(feature = "window")]
app.add_plugins(DefaultPlugins);  // Mac: full windowing & rendering

#[cfg(not(feature = "window"))]
app.add_plugins((
    ScheduleRunnerPlugin { ... },  // Pi: headless loop
    TimePlugin,                     // Pi: time tracking
));
```

**Key insight**: `MinimalPlugins` doesn't include a schedule runner. Headless apps need `ScheduleRunnerPlugin` to continuously run the Update schedule.

## Code Structure

```
src/
├── main.rs        # Platform-specific plugin setup (conditional compilation)
└── basic_demo.rs  # Platform-agnostic game logic (ECS components & systems)
```

**Bevy concepts demonstrated:**

- **Components**: Data attached to entities (`Person`, `Name`)
- **Resources**: Global data (`GreetTimer`)
- **Systems**: Functions that operate on components
  - Startup systems: Run once at app start
  - Update systems: Run every frame
- **Queries**: Access entities with specific components

**Benefits of this structure:**

- Game logic is platform-independent
- Easy to test headless mode on Mac
- Clear separation of concerns
- Explicit imports (no `use bevy::prelude::*`)

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
