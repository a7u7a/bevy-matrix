# WebGPU + Rust + LED Matrix [WIP]

This repo documents my AI-driven attempts at learning about modern rendering pipelines, Rust and cross-platform game development using the [Bevy game engine](https://github.com/bevyengine/bevy). 

## Examples

Each example runs on its own Bevy project:

- [`bevy_screen_demo`](/bevy_screen_demo/readme.md) <- Smoke test
- [`bevy_screen_3d_demo`](/bevy_screen_3d_demo/) <- Basic 3d scene demo(Boring cube but more interesting stuff coming up)
- [`bevy_screen_frag_shader_demo`](/) <- Full screen fragment shader demo

## About

After some time of looking for fun ways to run graphics code in an environment other than the browser, I finally came across [Bevy](https://github.com/bevyengine/bevy), a Rust based videogame engine which uses [Rust's WebGPU API](https://github.com/gfx-rs/wgpu). I really wanted to try it out, as its not only promising for building games but also suitable for building high-performance graphical interfaces. 

Hardware: 
- [Raspberry Pi Zero 2 W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/). Running on headless mode. Ideal for putting Rust to the test.
- [Adafruit RGB Matrix Bonnet for Raspberry Pi](https://www.adafruit.com/product/3211), compatible (almost) out of the box with the [Rust bindings](https://crates.io/crates/rpi-led-matrix) for [rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix) by Vince Pasquier.
- [64x64 px RGB LED Matrix display](https://www.adafruit.com/product/4732). Modest but fun.
- Custom 3d printed support for Rpi + screen (See photos below) 

I ran into a several challenges implementing this. Including:
- Compilation: The Pi Zero doesn't have enough RAM to run the Rust compiler. A Mac-to-Pi cross-compilation step was needed.
- Running the game in headless mode. A headless environment means no OS desktop environment -> no window -> no render target. 
- Assuming we could generate and obtain the rendered frame from Bevy:  How can we even extract it and paint it to the screen at a decent speed?
- Tweak the screen configuration to get glitch-free images.
- Figuring out platform specific dependencies, etc..

The goal was to create a proof-of-concept workflow for developing cross-platform games in resource-constrained systems.

I'm not heavily invested in rust, yet. However, I suspect there is promise in the potential for synergy between rust and webgpu that bevy could unlock.

## Build gallery

(Upcoming)



## Setup

### Running examples on Mac

Navigate to an example folder and run:

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

On the Mac: Use the deploy script.

```
$ ./deploy.sh
Building for Raspberry Pi...
Finished `release` profile [optimized] target(s) in 0.32s
Binary size: 66 MB
Copying to Pi...
bevy_screen_3d_demo                                                                                                                                                                                                                                          100%   65MB   4.6MB/s   00:14    
Done! Binary deployed to ayu@pi.local:~/
```

On the Pi:

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

