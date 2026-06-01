# Full screen metaballs shader example

Implemented using the (screen quad approach)[https://www.cginternals.com/en/blog/2018-01-10-screen-aligned-quads-and-triangles.html].


Edit the [`frag_shader.wgsl`](/bevy_screen_metaballs_demo/src/frag_shader.wgsl) to create custom shaders that run to the screen.

Compile on the Mac and send to Pi:

```bash
cd bevy_screen_metaballs_demo
./deploy.sh
```

On Pi

```bash
sudo ./bevy_screen_metaballs_demo
```