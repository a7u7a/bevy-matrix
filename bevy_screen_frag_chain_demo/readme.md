# Fragment shader demo: horizontally chained panels (128×32)

Two **32×64** (rows × cols per panel) RGB matrices daisy-chained horizontally → **128×32** logical resolution. Uses shared [`matrix_render`](../matrix_render) with explicit panel geometry and `chain_length = 2`.

Equivalent [rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix) flags:

```text
--led-rows=32 --led-cols=64 --led-chain=2
```

If the image is scrambled or colors are wrong, see the upstream README for `--led-multiplexing`, `--led-row-addr-type`, `--led-slowdown-gpio`, and wiring (`--led-gpio-mapping`).

Edit [`src/frag_shader.wgsl`](src/frag_shader.wgsl) for the effect. The shader tints the left and right halves differently so you can see the chain boundary.

## Build and run

**Mac (windowed preview):**

```bash
cd bevy_screen_frag_chain_demo
cargo run
```

**Cross-compile and copy to Pi** (adjust `HOST` in `deploy.sh`):

```bash
cd bevy_screen_frag_chain_demo
./deploy.sh
```

**On the Pi** (root required for GPIO):

```bash
sudo ./bevy_screen_frag_chain_demo
```

From the repo root you can also:

```bash
cargo run -p bevy_screen_frag_chain_demo
```
