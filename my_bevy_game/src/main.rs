// Explicit imports - only what we need
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

// Shared 3D scene setup (used by both render_demo and future matrix_render)
mod scene_setup;

// Render demo only used in window mode
#[cfg(feature = "window")]
mod render_demo;
#[cfg(feature = "window")]
use render_demo::RenderDemoPlugin;

// Matrix 3D rendering only used in headless mode
#[cfg(not(feature = "window"))]
mod matrix_render;
#[cfg(not(feature = "window"))]
use matrix_render::MatrixRenderPlugin;

// Target frame rate for headless mode
// Note: Window mode uses vsync (~60 FPS typically), headless uses fixed timestep
// 30 FPS provides good balance between smoothness and Pi Zero 2 W performance
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
        // Pi: Headless rendering with DefaultPlugins (following Bevy's headless_renderer.rs pattern)
        // Since we don't enable bevy_winit feature, WinitPlugin is not included in DefaultPlugins
        // Since we DO enable bevy_window feature, we need to manually add ScheduleRunnerPlugin
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    // Don't automatically exit due to having no windows
                    exit_condition: ExitCondition::DontExit,
                    ..Default::default()
                })
                // Configure the schedule runner with our target frame rate
                // Note: We use .set() to configure it, not .add_plugins() to avoid adding it twice
        )
        // Add ScheduleRunnerPlugin since DefaultPlugins doesn't include it when bevy_window is enabled
        .add_plugins(ScheduleRunnerPlugin {
            run_mode: bevy::app::RunMode::Loop {
                wait: Some(Duration::from_millis(FRAME_TIME_MS)),
            },
        });
    }

    app.add_plugins(BasicDemoPlugin);

    #[cfg(feature = "window")]
    app.add_plugins(RenderDemoPlugin);

    #[cfg(not(feature = "window"))]
    {
        // Matrix mode: 3D rendering to LED matrix
        app.add_plugins(MatrixRenderPlugin);
    }

    app.run();
}
