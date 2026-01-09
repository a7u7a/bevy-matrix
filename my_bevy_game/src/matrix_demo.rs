// Explicit imports - only what we need for this demo
use bevy::app::{App, Plugin, Startup, Update};
use bevy::ecs::system::Commands;
use bevy::prelude::{Res, ResMut, Resource};
use bevy::time::{Time, Timer, TimerMode};

/// Timer resource to control when we toggle the square size
#[derive(Resource, Debug)]
struct SquareTimer(Timer);

/// A minimal plugin demonstrating LED matrix integration with Bevy
/// This plugin will only actually write to the matrix when compiled with the "matrix" feature
/// on Linux (Raspberry Pi). On other platforms, it will just log that it would draw.
pub struct MatrixDemoPlugin;

impl Plugin for MatrixDemoPlugin {
    fn build(&self, app: &mut App) {
        // Insert timer resource
        app.insert_resource(SquareTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        
        // Add startup system to initialize the matrix
        app.add_systems(Startup, initialize_matrix);
        
        // Add update system to draw on the matrix every frame
        app.add_systems(Update, update_matrix_display);
    }
}

// Resource to hold the LED matrix
// Note: LedMatrix contains raw C pointers, but we're running in single-threaded headless mode
// so it's safe to mark our wrapper as Send/Sync
#[cfg(all(target_os = "linux", feature = "matrix"))]
use rpi_led_matrix::{LedMatrix, LedMatrixOptions, LedCanvas};

#[cfg(all(target_os = "linux", feature = "matrix"))]
#[derive(Resource)]
struct MatrixResource {
    matrix: LedMatrix,
    canvas: Option<LedCanvas>,
    square_size: i32,
}

// SAFETY: We only use this in single-threaded headless mode (ScheduleRunnerPlugin)
// The headless runner doesn't spawn threads, so it's safe to mark as Send/Sync
#[cfg(all(target_os = "linux", feature = "matrix"))]
unsafe impl Send for MatrixResource {}
#[cfg(all(target_os = "linux", feature = "matrix"))]
unsafe impl Sync for MatrixResource {}

fn initialize_matrix(mut commands: Commands) {
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    {
        println!("Initializing LED matrix...");
        
        let mut options = LedMatrixOptions::new();
        options.set_rows(64);
        options.set_cols(64);
        options.set_hardware_mapping("adafruit-hat-pwm");
        options.set_refresh_rate(true); // Equivalent to --led-show-refresh
        options.set_pwm_lsb_nanoseconds(130); // Default value for better display
        
        // Optional: adjust brightness (0-100)
        if let Err(e) = options.set_brightness(50) {
            eprintln!("Warning: Failed to set brightness: {}", e);
        }
        
        println!("Matrix configuration: 64x64, adafruit-hat-pwm mapping");
        
        let matrix = LedMatrix::new(Some(options), None)
            .expect("Failed to create LED matrix - check hardware connection and permissions");
        
        // Create the offscreen canvas ONCE during initialization
        let canvas = matrix.offscreen_canvas();
        
        commands.insert_resource(MatrixResource {
            matrix,
            canvas: Some(canvas),
            square_size: 8,
        });
        println!("LED matrix initialized successfully!");
    }

    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    {
        println!("Matrix feature not enabled - would initialize LED matrix");
    }
}

fn update_matrix_display(
    time: Res<Time>,
    mut timer: ResMut<SquareTimer>,
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    mut matrix_res: ResMut<MatrixResource>,
) {
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    {
        use rpi_led_matrix::LedColor;
        
        // Toggle square size every 2 seconds
        if timer.0.tick(time.delta()).just_finished() {
            matrix_res.square_size = if matrix_res.square_size == 8 { 16 } else { 8 };
            println!("Toggling square size to {}x{}", matrix_res.square_size, matrix_res.square_size);
        }

        // Take the canvas out of the Option temporarily
        if let Some(mut canvas) = matrix_res.canvas.take() {
            // Clear the canvas (black background)
            canvas.fill(&LedColor { red: 0, green: 0, blue: 0 });

            // Draw a red square with current size
            let red_color = LedColor { red: 255, green: 0, blue: 0 };
            
            for x in 0..matrix_res.square_size {
                for y in 0..matrix_res.square_size {
                    canvas.set(x, y, &red_color);
                }
            }

            // Swap the canvas to display it - double-buffering pattern
            // The returned canvas becomes our new offscreen canvas for the next frame
            matrix_res.canvas = Some(matrix_res.matrix.swap(canvas));
        }
    }

    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    {
        // Log once when timer triggers (to avoid spamming console)
        if timer.0.tick(time.delta()).just_finished() {
            println!("Matrix feature not enabled - would toggle square size");
        }
    }
}

