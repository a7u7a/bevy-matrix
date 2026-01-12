// TEST 3: DefaultPlugins + Minimal GPU Rendering
// This test adds simple GPU rendering to isolate if GPU→CPU transfer causes flicker
// GPU work is MINIMAL: just clear color (no 3D scene)

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

// GPU rendering only used in headless mode
#[cfg(not(feature = "window"))]
mod gpu_render;
#[cfg(not(feature = "window"))]
use gpu_render::GpuRenderPlugin;

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
        println!("Running in headless mode with GPU rendering (fixed timestep: {}ms)", FRAME_TIME_MS);
        println!("TEST: Adding minimal GPU rendering (clear color) to check if GPU→CPU causes flicker");
        
        // Using DefaultPlugins (proven to work in Test 2)
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
    app.add_plugins(GpuRenderPlugin);

    app.run();
}

