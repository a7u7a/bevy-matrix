// Explicit imports - only what we need
use bevy::app::App;

// For fully explicit imports, see: https://docs.rs/bevy/latest/bevy/struct.DefaultPlugins.html
#[cfg(feature = "window")]
use bevy::DefaultPlugins;

// Headless mode plugins - minimal set for console-only operation
#[cfg(not(feature = "window"))]
use bevy::app::ScheduleRunnerPlugin;
#[cfg(not(feature = "window"))]
use bevy::time::TimePlugin;
#[cfg(not(feature = "window"))]
use std::time::Duration;

// Declare the modules
mod basic_demo;
use basic_demo::BasicDemoPlugin;

// Render demo only used in window mode
#[cfg(feature = "window")]
mod render_demo;
#[cfg(feature = "window")]
use render_demo::RenderDemoPlugin;

// Matrix demo only used in headless mode
#[cfg(not(feature = "window"))]
mod matrix_demo;
#[cfg(not(feature = "window"))]
use matrix_demo::MatrixDemoPlugin;

// Target frame rate for headless mode
// Note: Window mode uses vsync (~60 FPS typically), headless uses fixed timestep
// These won't be perfectly synchronized, but should be close enough for game logic
#[cfg(not(feature = "window"))]
const TARGET_FPS: u64 = 60;
#[cfg(not(feature = "window"))]
const FRAME_TIME_MS: u64 = 1000 / TARGET_FPS;

fn main() {
    let mut app = App::new();

    // Platform-specific plugin configuration
    #[cfg(feature = "window")]
    {
        println!("Running in windowed mode");
        // Mac: DefaultPlugins includes windowing, rendering, input, etc.
        // Runs at window refresh rate (typically 60 FPS via vsync)
        app.add_plugins(DefaultPlugins);
    }

    #[cfg(not(feature = "window"))]
    {
        println!("Running in headless mode (fixed timestep: {}ms)", FRAME_TIME_MS);
        // Pi: Minimal plugins for headless operation
        // Window mode uses vsync-driven event loop (WinitPlugin)
        // Headless mode uses fixed timestep loop (ScheduleRunnerPlugin)
        app.add_plugins((
            ScheduleRunnerPlugin {
                run_mode: bevy::app::RunMode::Loop {
                    wait: Some(Duration::from_millis(FRAME_TIME_MS)),
                },
            },
            TimePlugin,
        ));
    }

    app.add_plugins(BasicDemoPlugin);

    #[cfg(feature = "window")]
    app.add_plugins(RenderDemoPlugin);

    #[cfg(not(feature = "window"))]
    app.add_plugins(MatrixDemoPlugin);

    app.run();
}
