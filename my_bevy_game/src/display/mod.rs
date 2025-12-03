// DisplayBackend trait + Resource wrapper

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct DisplayResource(pub Box<dyn DisplayBackend>);

// Safety: We ensure single-threaded access through Bevy's ECS
unsafe impl Send for DisplayResource {}
unsafe impl Sync for DisplayResource {}

pub trait DisplayBackend {
    fn new() -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn write_frame(&mut self, pixels: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(feature = "window")]
pub mod window;
#[cfg(feature = "window")]
pub use window::WindowBackend as Backend;

#[cfg(feature = "matrix")]
pub mod matrix;
#[cfg(feature = "matrix")]
pub use matrix::MatrixBackend as Backend;
