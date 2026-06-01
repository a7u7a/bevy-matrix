# Full screen fragment shader example

Implemented using the (screen quad approach)[https://www.cginternals.com/en/blog/2018-01-10-screen-aligned-quads-and-triangles.html].


Edit the [`frag_shader.wgsl`](/examples/noise_shader/src/frag_shader.wgsl) to create custom shaders that run to the screen.


Compile on the Mac and send to Pi:

```bash
cd examples/noise_shader
./deploy.sh
```

On Pi

```bash
sudo ./noise_shader
```