use super::DisplayBackend;

pub struct WindowBackend {
    width: u32,
    height: u32,
}

impl DisplayBackend for WindowBackend {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            width: 64,
            height: 64,
        })
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn write_frame(&mut self, _pixels: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Bevy handles window rendering automatically
        Ok(())
    }
}
