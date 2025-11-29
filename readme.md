# Learning WGPU

The goal of this repository is to tryout [wgpu](https://github.com/gfx-rs/wgpu?tab=readme-ov-file) and document my learning process.

## Hardware

Aiming to compile and run wgpu on a Raspberry Pi Zero W and print the wgpu output to a LED Matrix screen (like [this one](https://www.adafruit.com/product/4732)) using the [Adafruit RGB Matrix Bonnet for Raspberry Pi](https://www.adafruit.com/product/3211).

https://sotrh.github.io/learn-wgpu/#what-is-wgpu

## Trying out Bevy

(in progress, documenting)
(installed rust, cargo and rust analyzer)

```bash
cargo new my_bevy_game
cd my_bevy_game
```

Install bevy

```bash
cargo add bevy
```

Add this to `main.rs`:

```rs
use bevy::prelude::*;

fn main() {
    App::new().run();
}
```

Run:

```bash
cargo run
```

## Piping data to LED matrix

(pending)
