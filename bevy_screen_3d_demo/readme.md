# Test 5: GPU Rendering with 3D Scene (Rotating Cube)

## Goal
Test if full 3D scene rendering causes flicker by adding 3D complexity to the proven working GPU pipeline from Test 4.

## Hypothesis
Something about 3D rendering (PBR pipeline, lighting, mesh complexity) might cause the flicker observed in `my_bevy_game`.

## Implementation
Based on Test 4's working foundation:
- ✅ FrameBuffer pre-buffer strategy
- ✅ `set_brightness(50)`
- ✅ Always swap every frame (no frame_changed check)
- ✅ Double buffering pattern
- **NEW**: Full 3D scene with rotating red cube
- **NEW**: Camera3d with PBR lighting

## Expected Results
- **No flicker**: 3D rendering works fine with FrameBuffer → Root cause is frame_changed check
- **Flickers**: 3D rendering itself causes issues → Need to investigate PBR/lighting complexity

## Build and Deploy
```bash
./deploy.sh
```

## Run on Pi
```bash
sudo ./bevy_screen_3d_demo
```

