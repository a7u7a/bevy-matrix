// Explicit imports - only what we need for this demo
use bevy::app::{App, Plugin, PostUpdate, Startup};
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
        app.insert_resource(SquareTimer(Timer::from_seconds(0.25, TimerMode::Repeating)));
        
        // Add startup system to initialize the matrix
        app.add_systems(Startup, initialize_matrix);
        
        // IMPORTANT: Use PostUpdate instead of Update
        // PostUpdate runs AFTER rendering is complete, which is when we want to
        // display the frame on the LED matrix. This ensures we're showing the most
        // recent rendered frame.
        // 
        // Schedule order: Update → Rendering → PostUpdate
        // 
        // For future 3D rendering integration, the GPU→CPU frame extraction will
        // also happen in PostUpdate, so this system will run after frame data is available.
        app.add_systems(PostUpdate, update_matrix_display);
    }
}

// Resource to hold the LED matrix
// 
// THREAD SAFETY CONSIDERATIONS:
// - The rpi-led-matrix C library contains raw pointers and may not be inherently thread-safe
// - We mark this as Send/Sync because Bevy requires Resources to be Send/Sync
// - SAFETY RATIONALE:
//   1. This resource is ONLY accessed in PostUpdate schedule via ResMut (exclusive access)
//   2. ResMut provides Rust's exclusive borrowing guarantees at runtime
//   3. Only ONE system accesses this resource, so no parallel access is possible
//   4. Even with multi_threaded enabled, Bevy's system scheduler ensures exclusive access
// 
// ALTERNATIVE APPROACHES (if issues arise):
// - Use ResMut<NonSend<MatrixResource>> to force main-thread-only access
// - Wrap in Mutex for explicit locking (adds overhead)
// - Use channels to send frame data to a dedicated matrix thread
#[cfg(all(target_os = "linux", feature = "matrix"))]
use rpi_led_matrix::{LedMatrix, LedMatrixOptions, LedCanvas};

#[cfg(all(target_os = "linux", feature = "matrix"))]
#[derive(Resource)]
pub struct MatrixResource {
    pub matrix: LedMatrix,
    pub canvas: Option<LedCanvas>,
    pub square_size: i32,
}

// SAFETY: See detailed comment above. This is safe because:
// 1. Exclusive access guaranteed by ResMut
// 2. Only one system accesses this resource
// 3. No concurrent access possible due to Bevy's scheduling
#[cfg(all(target_os = "linux", feature = "matrix"))]
unsafe impl Send for MatrixResource {}
#[cfg(all(target_os = "linux", feature = "matrix"))]
unsafe impl Sync for MatrixResource {}

fn initialize_matrix(
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    mut commands: Commands,
    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    _commands: Commands,
) {
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
        
        // Toggle square size periodically
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

            // DOUBLE BUFFERING: Required by rpi-led-matrix library
            // The .swap() call does two things:
            // 1. Displays the canvas we just drew to the LED matrix (atomic flip)
            // 2. Returns the previous display buffer for us to draw the next frame
            // 
            // This prevents tearing and flicker on the LED display - it's a hardware-level
            // optimization, not a Bevy pattern. Without it, you'd see partial frames.
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

