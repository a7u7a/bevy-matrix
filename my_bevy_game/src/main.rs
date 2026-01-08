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

// Declare the module
mod basic_demo;
use basic_demo::BasicDemoPlugin;

// Target 60 FPS on both platforms for consistency
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
        println!("Running in headless mode");
        // Pi: Minimal plugins for headless operation
        // Manually run at 60 FPS to match window mode behavior
        app.add_plugins((
            ScheduleRunnerPlugin {
                run_mode: bevy::app::RunMode::Loop {
                    wait: Some(Duration::from_millis(FRAME_TIME_MS)),
                },
            },
            TimePlugin,
        ));
    }

    // Add our game logic (works on both platforms)
    app.add_plugins(BasicDemoPlugin);

    app.run();
}
