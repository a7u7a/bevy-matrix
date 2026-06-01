// Bevy 3D Demo - GPU rendering to LED matrix (Pi) or window (Mac)
//
// Run modes:
//   - `cargo run` (Mac): Opens window with rotating red cube
//   - `cargo build --features matrix --no-default-features` (Pi): Headless LED matrix rendering

use bevy::app::App;
use bevy::DefaultPlugins;

// Shared modules (both platforms)
mod scene_setup;
mod uv_material;
use scene_setup::ScenePlugin;

// Headless mode plugins
#[cfg(not(feature = "window"))]
use bevy::app::{PluginGroup, ScheduleRunnerPlugin};
#[cfg(not(feature = "window"))]
use bevy::window::{ExitCondition, WindowPlugin};
#[cfg(not(feature = "window"))]
use std::time::Duration;

#[cfg(not(feature = "window"))]
use bevy_rgb_matrix::{MatrixConfig, MatrixRenderPlugin};

// ECS demo (optional, for testing)
mod basic_demo;
use basic_demo::BasicDemoPlugin;

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
        println!("Running in headless mode ({}ms frame time)", FRAME_TIME_MS);

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            exit_condition: ExitCondition::DontExit,
            ..Default::default()
        }))
        .add_plugins(ScheduleRunnerPlugin {
            run_mode: bevy::app::RunMode::Loop {
                wait: Some(Duration::from_millis(FRAME_TIME_MS)),
            },
        });
    }

    // Shared 3D scene (works on both platforms)
    app.add_plugins(ScenePlugin);

    // ECS demo
    app.add_plugins(BasicDemoPlugin);

    #[cfg(not(feature = "window"))]
    app.add_plugins(MatrixRenderPlugin {
        config: MatrixConfig {
            pwm_lsb_nanoseconds: 200,
            ..Default::default()
        },
    });

    app.run();
}
