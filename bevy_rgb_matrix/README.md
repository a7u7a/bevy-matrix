# bevy_rgb_matrix

Bevy plugin that renders a marked camera to a GPU framebuffer, copies pixels to the CPU, and (with the `matrix` feature on Linux) drives an RGB LED panel through [rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix).

**Bevy version:** 0.17.x

## Features

| Feature | Description |
|---------|-------------|
| *(default)* | GPU render target + CPU readback only (no hardware). Useful for headless dev on a laptop. |
| `matrix` | Linux only: links `rpi-led-matrix` and uploads frames to the physical panel. |

## Quick start (Raspberry Pi / headless)

1. Add the dependency with the `matrix` feature:

```toml
bevy_rgb_matrix = { version = "0.1", features = ["matrix"] }
```

2. Configure a headless Bevy app (no window) with rendering enabled. You need `RenderPlugin`, a scene, and a fixed timestep. See the [rotating_3d_cube](https://github.com/a7u7a/rpi-wgpu/tree/main/examples/rotating_3d_cube) example in this repo.

3. Mark exactly one camera with `MatrixCamera` and add the plugin:

```rust
use bevy_rgb_matrix::{MatrixCamera, MatrixConfig, MatrixRenderPlugin};

app.add_plugins(MatrixRenderPlugin {
    config: MatrixConfig::default(),
});
// On your camera entity:
commands.spawn((Camera3d::default(), MatrixCamera));
```

## System dependencies (`matrix` feature)

On the Pi you must build and link [hzeller/rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix):

```bash
git clone https://github.com/hzeller/rpi-rgb-led-matrix.git
cd rpi-rgb-led-matrix
make -C lib
sudo make install  # or set RPI_LED_MATRIX_DIR / library search path for your setup
```

The `rpi-led-matrix` crate expects `librgbmatrix` to be available at link time. GPIO access typically requires `sudo` when running your binary.

### Hardware mapping

Set `MatrixConfig::hardware_mapping` to match your wiring. Common values are documented in the [hzeller hardware mappings](https://github.com/hzeller/rpi-rgb-led-matrix#types-of-displays) section (e.g. `adafruit-hat-pwm`, `regular`, `adafruit-hat`).

### Audio / PWM

The BCM2835 PWM used by many HATs conflicts with the Pi audio driver. Disable audio (`dtparam=audio=off` in `/boot/firmware/config.txt`) and blacklist `snd_bcm2835` if you see initialization errors. More notes live in the [workspace readme](https://github.com/a7u7a/rpi-wgpu/blob/main/readme.md#troubleshooting).

## Local development without hardware

Build without `matrix` on macOS or Linux to exercise the GPU→CPU path only:

```bash
cargo build -p your_app --no-default-features
```

Frames are read back but not sent to a panel.

## Examples

Full Bevy apps live in the parent repository:

- [blink](https://github.com/a7u7a/rpi-wgpu/tree/main/examples/blink) — smoke test
- [rotating_3d_cube](https://github.com/a7u7a/rpi-wgpu/tree/main/examples/rotating_3d_cube) — 3D scene
- [noise_shader](https://github.com/a7u7a/rpi-wgpu/tree/main/examples/noise_shader) — fragment shader
- [metaballs_shader](https://github.com/a7u7a/rpi-wgpu/tree/main/examples/metaballs_shader) — metaballs shader

## Publishing (maintainers)

From the workspace root:

```bash
cargo publish -p bevy_rgb_matrix --dry-run
cargo publish -p bevy_rgb_matrix
```

Example crates in `examples/` are `publish = false` and are not uploaded.
