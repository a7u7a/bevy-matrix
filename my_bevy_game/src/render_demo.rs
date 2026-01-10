// Explicit imports - only what we need for 3D rendering
use bevy::app::{App, Plugin, Startup};
use bevy::ecs::system::Query;
use bevy::window::{Window, WindowResolution};

// Import shared scene setup
use crate::scene_setup::{setup_3d_scene, RENDER_WIDTH, RENDER_HEIGHT};

/// A minimal plugin demonstrating 3D scene rendering in a window
/// This plugin configures the window to 64x64 and uses the shared 3D scene
pub struct RenderDemoPlugin;

impl Plugin for RenderDemoPlugin {
    fn build(&self, app: &mut App) {
        // Add startup systems to configure window and set up the 3D scene
        app.add_systems(Startup, (configure_window, setup_3d_scene));
    }
}

/// Configure the window to match LED matrix resolution (64x64)
fn configure_window(mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.single_mut() {
        window.resolution = WindowResolution::new(RENDER_WIDTH, RENDER_HEIGHT);
        window.title = "Bevy LED Matrix - 64x64".to_string();
        println!("Window configured: {}x{}", RENDER_WIDTH, RENDER_HEIGHT);
    }
}

