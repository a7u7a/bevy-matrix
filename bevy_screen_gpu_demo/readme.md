# Test 3: DefaultPlugins + Minimal GPU Rendering

## Purpose

This test isolates whether GPU rendering + CPU data transfer causes the LED matrix flicker.

## Changes from Test 2

**Test 2**: `DefaultPlugins` + direct matrix drawing → NO FLICKER ✅

**Test 3 (this)**: `DefaultPlugins` + GPU rendering + matrix display

## GPU Rendering

**MINIMAL GPU work**:
- Creates 64x64 offscreen render target
- 2D camera with default clear (no 3D geometry, no complex shaders)
- Copies GPU texture → CPU buffer
- Displays on matrix

This is the simplest possible GPU rendering test.

## Test Matrix

| Test | Plugins | GPU Rendering | Result |
|------|---------|---------------|--------|
| 1 | Minimal | None | ✅ NO FLICKER |
| 2 | DefaultPlugins | None | ✅ NO FLICKER |
| 3 | DefaultPlugins | **Minimal (clear color)** | ❓ |
| 4 | DefaultPlugins | Full 3D scene | ❌ FLICKERS |

## Expected Outcomes

| Result | Conclusion |
|--------|------------|
| **Flickers** | GPU→CPU data transfer causes flicker |
| **No flicker** | Simple GPU work is fine → Issue is in complex 3D rendering |

## Build & Deploy

```bash
./deploy.sh
```

On Pi:
```bash
sudo ./bevy_screen_gpu_demo
```

## What to Watch For

- Screen should show GPU-rendered content (likely dark/black since nothing is drawn)
- Any black flashes/flickering?
- Does it feel different from Test 2?

