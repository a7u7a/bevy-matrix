# Render bevy scenes to a LED matrix display

Fuses together `bevy` and `rpi-led-matrix` to render real time graphics to the screen using a Raspberry Pi.

## Crate

The reusable Bevy plugin lives in [`bevy_rgb_matrix/`](bevy_rgb_matrix/). After publishing:

- [crates.io/crates/bevy_rgb_matrix](https://crates.io/crates/bevy_rgb_matrix)
- [docs.rs/bevy_rgb_matrix](https://docs.rs/bevy_rgb_matrix)

Publish from the workspace root (examples are not uploaded):

```bash
cargo publish -p bevy_rgb_matrix --dry-run
cargo publish -p bevy_rgb_matrix
```

- Supports running in headless mode
- Run WebGPU shaders and full 3D scenes at 60fps on low-end hardware (Raspberry Pi Zero W 2)
- Away from your LED-Matrix? Run the exact same scene in your laptop!
- Includes MacOS -> raspberry pi cross-compilation examples
- Multi-panel support

## Pending

- Make example that uses chained panels
- Make a stress test example

## Performance

(Upcoming)

## Build gallery

(Upcoming)

## Examples

Each example is a Bevy project under [`examples/`](examples/):

- [`blink`](examples/blink/readme.md) — smoke test
- [`rotating_3d_cube`](examples/rotating_3d_cube/readme.md) — basic 3D scene
- [`noise_shader`](examples/noise_shader/readme.md) — full-screen fragment shader
- [`metaballs_shader`](examples/metaballs_shader/readme.md) — metaballs fragment shader

## Setup

### Running examples on Mac

Navigate to an example folder (e.g. `cd examples/blink`) and run:

```bash
cargo run --features window
# or just:
cargo run
```

**Test headless mode locally** (simulates Pi environment):

```bash
cargo run --no-default-features --features matrix
```

**Development profiles:**

```bash
# Standard dev build (optimized for iteration speed)
# - Dependencies: opt-level 3 (slow first compile, fast runtime)
cargo run

# Ultra-fast compilation for quick syntax checks
cargo build --profile dev-fast --features window

# Release build (maximum optimization)
cargo build --release
```

The dev profile balances compile time and runtime performance. First compile is slow (dependencies), but subsequent builds are fast since dependencies are cached.

### Running examples on the Raspberry Pi

#### The cross-compilation setup

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

On the Mac: Use the deploy script from the example directory (e.g. `cd examples/rotating_3d_cube`).

```
$ ./deploy.sh
Building for Raspberry Pi...
Finished `release` profile [optimized] target(s) in 0.32s
Binary size: 66 MB
Copying to Pi...
rotating_3d_cube                                                                                                                                                                                                                                          100%   65MB   4.6MB/s   00:14    
Done! Binary deployed to ayu@pi.local:~/
```

On the Pi:

```bash
sudo ./rotating_3d_cube
```

## Tips

[VSCode tasjs](.vscode/tasks.json) make dev work a lil easier.

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

### Intermittent flicker on the LED matrix

**Symptom:** Occasional 1-pixel-wide white horizontal streaks (random Y position, no
more than a handful at once), appearing for a few frames then vanishing. Tends to
appear on the second or third launch after boot rather than the first, and clears
after a cold reboot.

**What was ruled out** (Pi Zero 2 W + Adafruit RGB Matrix Bonnet + 64x64 panel,
`gpio_slowdown=2`, `pwm_bits=7`, `pwm_lsb_nanoseconds=200`, `brightness=30`,
`adafruit-hat-pwm`, hardware pulsing on):

- App-side data path: GPU readback, padded-row stride handling, dither, and bulk
  FFI upload produced consistent pixel data in clean and flickering runs.
- System state: CPU governor `performance` at 1 GHz, core temp around 47 C,
  `throttled=0x0`, `voltage=1.2563V`, no `snd_bcm2835` module loaded, and no
  conflicting `pigpio`/`pulseaudio`/`alsa`/`jack`/`gpio` processes.
- Timing dials: reducing `pwm_bits` to 5 or 6, raising `gpio_slowdown` to 4 or 5,
  and lowering `brightness` to 15-20 darkens the panel below visible threshold
  without resolving flicker.

**Hypothesis that best fits the observed pattern**

State outside the process: the BCM2835 hardware PWM / DMA peripherals
(driven by the `GPIO4 <-> GPIO18` jumper for the bonnet "quality mod") can retain
clock/divider state between launches. A previous process may leave them in a
borderline configuration that produces OE timing jitter on the next launch. A cold
reboot fully resets the peripheral and restores clean output.

**Mitigations to try first if it returns**

- Cold-reboot the Pi (`sudo shutdown -h now`, pull power for a few seconds, reconnect).
- Confirm `snd_bcm2835` is still blacklisted and `dtparam=audio=off` is in
  `/boot/firmware/config.txt`.
- Make sure no other process is touching `/dev/gpiomem`, `pwmchip0`, or audio when
  the demo starts.
- Re-seat the bonnet on the Pi header and the ribbon cable on the panel side;
  intermittent contacts can produce the same visual signature.

