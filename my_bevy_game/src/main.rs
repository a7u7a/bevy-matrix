// Explicit imports - only what we need
use bevy::app::App;

// Only import these for headless mode
#[cfg(not(feature = "window"))]
use bevy::app::ScheduleRunnerPlugin;
#[cfg(not(feature = "window"))]
use bevy::time::TimePlugin;
#[cfg(not(feature = "window"))]
use std::time::Duration;

// Declare the module
mod basic_demo;
use basic_demo::BasicDemoPlugin;

fn main() {
    let mut app = App::new();

    // Platform-specific plugin configuration
    #[cfg(feature = "window")]
    {
        // Mac: Use DefaultPlugins for windowing and rendering
        use bevy::DefaultPlugins;
        app.add_plugins(DefaultPlugins);
    }

    #[cfg(not(feature = "window"))]
    {
        // Pi: Use minimal plugins for headless operation
        app.add_plugins((
            // Run the app in a loop with a fixed timestep
            ScheduleRunnerPlugin {
                run_mode: bevy::app::RunMode::Loop {
                    wait: Some(Duration::from_millis(100)),
                },
            },
            // Time tracking for our timer
            TimePlugin,
        ));
    }

    // Add our game logic (works on both platforms)
    app.add_plugins(BasicDemoPlugin);

    app.run();
}
