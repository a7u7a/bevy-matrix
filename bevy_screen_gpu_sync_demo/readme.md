# Test 4: GPU Rendering with Pre-buffer Strategy

## Goal
Test if pre-buffering GPU frame data eliminates flicker by ensuring atomic canvas updates.

## Hypothesis
The flicker in Test 3 was caused by the matrix refresh thread seeing intermediate states during the pixel-by-pixel canvas drawing loop. By copying GPU data to a stable buffer first, then drawing from that buffer, we ensure all pixels come from the same frame.

## Implementation
1. **FrameBuffer Resource**: Stable buffer to hold frame data
2. **Two-phase drawing**: 
   - Phase 1: Fast memcpy from GPU data to FrameBuffer (~1µs)
   - Phase 2: Draw from stable FrameBuffer to canvas
3. **Animated test pattern**: Moving red square to observe sync under motion

## Expected Results
- **No flicker**: Pre-buffering solves the timing issue → Use this approach in main project
- **Still flickers**: Issue is deeper (canvas.set() loop itself too slow) → Need different solution

## Build and Deploy
```bash
./deploy.sh
```

## Run on Pi
```bash
sudo ./bevy_screen_gpu_sync_demo
```

