# Test 2: DefaultPlugins + Direct Matrix Drawing

## Purpose

This test isolates whether using `DefaultPlugins` causes the LED matrix flicker.

## Changes from Working Demo (bevy_screen_demo)

**ONLY ONE CHANGE**:

- `bevy_screen_demo`: Uses minimal plugins (`ScheduleRunnerPlugin` + `TimePlugin`) ✅ NO FLICKER
- **This test**: Uses `DefaultPlugins` (headless config) + `ScheduleRunnerPlugin` ❓

## Matrix Drawing

Matrix drawing code is **IDENTICAL** to working demo:

- Direct drawing with `canvas.fill()` + `canvas.set()`
- Same double buffering pattern (create canvas once, swap continuously)
- Same 60 FPS rate
- Same red square animation

## Expected Outcomes

| Result         | Conclusion                                                               |
| -------------- | ------------------------------------------------------------------------ |
| **Flickers**   | DefaultPlugins interferes with matrix refresh → CPU contention confirmed |
| **No flicker** | DefaultPlugins is fine → Issue must be in GPU rendering pipeline         |

## Build & Deploy

```bash
./deploy.sh
```

On Pi:

```bash
sudo ./bevy_screen_default_plugins_demo
```

## What to Watch For

- Does the red square animate smoothly? (8x8 ↔ 16x16 every 2 seconds)
- Any black flashes during updates?
- Does CPU usage differ from minimal plugin version?

## Test run results

- No flicker, or difference with `bevy_screen_demo`
