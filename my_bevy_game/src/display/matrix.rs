use super::DisplayBackend;
use rpi_led_matrix::{LedCanvas, LedColor, LedMatrix};
use std::mem;

pub struct MatrixBackend {
    matrix: LedMatrix,
    canvas: Option<LedCanvas>,
}

// Safety: The matrix will only be accessed from Bevy's main thread
unsafe impl Send for MatrixBackend {}
unsafe impl Sync for MatrixBackend {}

impl DisplayBackend for MatrixBackend {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let matrix = LedMatrix::new(None, None)?;
        let canvas = matrix.offscreen_canvas();
        Ok(Self {
            matrix,
            canvas: Some(canvas),
        })
    }

    fn width(&self) -> u32 {
        64
    }

    fn height(&self) -> u32 {
        64
    }

    fn write_frame(&mut self, pixels: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut canvas) = self.canvas.take() {
            for y in 0..64 {
                for x in 0..64 {
                    let idx = ((y * 64 + x) * 3) as usize;
                    let color = LedColor {
                        red: pixels[idx],
                        green: pixels[idx + 1],
                        blue: pixels[idx + 2],
                    };
                    canvas.set(x as i32, y as i32, &color);
                }
            }
            self.canvas = Some(self.matrix.swap(canvas));
        }
        Ok(())
    }
}
