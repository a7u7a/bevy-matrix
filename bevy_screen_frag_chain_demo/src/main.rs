// Bevy fragment shader demo: two 32×64 panels chained horizontally (128×32 logical).
//
// Run modes:
//   - `cargo run` (Mac): windowed preview
//   - `cargo build --features matrix --no-default-features` (Pi): headless LED matrix

use bevy::app::App;
use bevy::DefaultPlugins;

mod frag_shader;
use frag_shader::FragShaderPlugin;

#[cfg(not(feature = "window"))]
use bevy::app::{PluginGroup, ScheduleRunnerPlugin};
#[cfg(not(feature = "window"))]
use bevy::window::{ExitCondition, WindowPlugin};
#[cfg(not(feature = "window"))]
use std::time::Duration;

#[cfg(not(feature = "window"))]
use matrix_render::{MatrixConfig, MatrixRenderPlugin};

#[cfg(not(feature = "window"))]
const TARGET_FPS: u64 = 60;
#[cfg(not(feature = "window"))]
const FRAME_TIME_MS: u64 = 1000 / TARGET_FPS;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "window")]
    {
        println!("Running in windowed mode (128×32 chain preview)");
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

    app.add_plugins(FragShaderPlugin);

    #[cfg(not(feature = "window"))]
    app.add_plugins(MatrixRenderPlugin {
        config: MatrixConfig {
            render_width: 128,
            render_height: 32,
            panel_rows: Some(32),
            panel_cols: Some(64),
            chain_length: 2,
            parallel: 1,
            pwm_lsb_nanoseconds: 100,
            pwm_dither_bits: Some(2),
            ..Default::default()
        },
    });

    app.run();
}
