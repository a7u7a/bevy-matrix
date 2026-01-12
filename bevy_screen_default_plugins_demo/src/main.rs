// TEST 2: DefaultPlugins + Direct Matrix Drawing
// This test isolates whether DefaultPlugins alone causes flicker
// Matrix drawing is IDENTICAL to working bevy_screen_demo

use bevy::app::App;
use bevy::DefaultPlugins;

// Headless mode plugins
#[cfg(not(feature = "window"))]
use bevy::app::{PluginGroup, ScheduleRunnerPlugin};
#[cfg(not(feature = "window"))]
use bevy::window::{ExitCondition, WindowPlugin};
#[cfg(not(feature = "window"))]
use std::time::Duration;

// Declare the modules
mod basic_demo;
use basic_demo::BasicDemoPlugin;

// Matrix demo only used in headless mode
#[cfg(not(feature = "window"))]
mod matrix_demo;
#[cfg(not(feature = "window"))]
use matrix_demo::MatrixDemoPlugin;

// Target frame rate for headless mode
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
        app.add_plugins(DefaultPlugins);
    }

    #[cfg(not(feature = "window"))]
    {
        println!("Running in headless mode with DefaultPlugins (fixed timestep: {}ms)", FRAME_TIME_MS);
        println!("TEST: Replacing minimal plugins with DefaultPlugins to isolate flicker cause");
        
        // KEY CHANGE: Using DefaultPlugins instead of just ScheduleRunnerPlugin + TimePlugin
        // This adds all Bevy subsystems including RenderApp, even in headless mode
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..Default::default()
                })
        )
        .add_plugins(ScheduleRunnerPlugin {
            run_mode: bevy::app::RunMode::Loop {
                wait: Some(Duration::from_millis(FRAME_TIME_MS)),
            },
        });
    }

    app.add_plugins(BasicDemoPlugin);

    #[cfg(not(feature = "window"))]
    app.add_plugins(MatrixDemoPlugin);

    app.run();
}

