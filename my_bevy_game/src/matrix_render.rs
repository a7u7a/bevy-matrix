// Headless 3D rendering for LED Matrix
// Based on Bevy's headless_renderer.rs example
// This module handles:
// 1. Rendering 3D scene to an offscreen texture
// 2. Copying texture from GPU to CPU via buffer
// 3. Sending frame data to main world via channel

use bevy::app::{App, Plugin, PostStartup, PostUpdate, Startup};
use bevy::asset::{Assets, Handle};
use bevy::camera::RenderTarget;
use bevy::ecs::component::Component;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::ecs::world::World;
use bevy::image::Image;
use bevy::prelude::{Camera, Deref, DerefMut, Resource};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel};
use bevy::render::render_resource::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode, PollType,
    TexelCopyBufferInfo, TexelCopyBufferLayout, TextureFormat, TextureUsages,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp};
use crossbeam_channel::{Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::scene_setup::{setup_3d_scene, RENDER_HEIGHT, RENDER_WIDTH};

/// Set to true to bypass GPU rendering and display a static test pattern
/// This helps isolate whether flickering is caused by GPU pipeline or matrix library
const TEST_PATTERN_MODE: bool = true;

// LED Matrix types (only available on Linux with matrix feature)
#[cfg(all(target_os = "linux", feature = "matrix"))]
use rpi_led_matrix::{LedCanvas, LedMatrix, LedMatrixOptions};

/// Resource to hold the LED matrix hardware and offscreen canvas for double buffering
/// This is created during startup and used to display rendered frames
#[cfg(all(target_os = "linux", feature = "matrix"))]
#[derive(Resource)]
pub struct MatrixResource {
    pub matrix: LedMatrix,
    /// Offscreen canvas for double buffering - we draw here, then swap atomically
    /// Using Option allows us to take() ownership without creating a placeholder canvas
    pub offscreen_canvas: Option<LedCanvas>,
    /// Cache of last valid frame data - used to maintain display if GPU frame not ready
    /// This ensures we always have something to display and can swap every frame
    pub last_valid_frame: Option<Vec<u8>>,
}

// SAFETY: MatrixResource is only accessed via ResMut (exclusive access)
// Bevy's scheduler ensures no concurrent access even in multi_threaded mode
#[cfg(all(target_os = "linux", feature = "matrix"))]
unsafe impl Send for MatrixResource {}
#[cfg(all(target_os = "linux", feature = "matrix"))]
unsafe impl Sync for MatrixResource {}

/// Main plugin for headless 3D rendering to LED matrix
pub struct MatrixRenderPlugin;

impl Plugin for MatrixRenderPlugin {
    fn build(&self, app: &mut App) {
        // Initialize LED matrix hardware first
        app.add_systems(Startup, initialize_matrix);
        
        // Setup 3D scene
        app.add_systems(Startup, setup_3d_scene);
        
        // Setup render target in PostStartup (after render plugins are initialized)
        // This ensures RenderDevice and other render resources are available
        app.add_systems(PostStartup, setup_render_target);
        
        // Add the image copy plugin for GPU→CPU frame extraction
        app.add_plugins(ImageCopyPlugin);
        
        // Add system to receive frame data and update matrix
        // This runs in PostUpdate after rendering is complete
        app.add_systems(PostUpdate, receive_and_display_frame);
    }
}

/// Channel receiver in main world for frame data from render world
#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u8>>);

/// Channel sender in render world to send frame data to main world
#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u8>>);

/// Initialize the LED matrix hardware
/// This runs once at startup to configure the matrix
fn initialize_matrix(mut commands: Commands) {
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    {
        println!("Initializing LED matrix...");
        
        let mut options = LedMatrixOptions::new();
        options.set_rows(64);
        options.set_cols(64);
        options.set_hardware_mapping("adafruit-hat-pwm");
        options.set_refresh_rate(true);
        // Using EXACT same settings as working demo - no extra PWM/brightness settings
        
        println!("Matrix configuration: 64x64, adafruit-hat-pwm mapping (matching working demo)");
        
        let matrix = LedMatrix::new(Some(options), None)
            .expect("Failed to create LED matrix - check hardware connection and permissions");
        
        // Create offscreen canvas for double buffering
        // This allows us to draw the entire frame invisibly, then swap atomically
        let offscreen_canvas = matrix.offscreen_canvas();
        println!("Created offscreen canvas for double buffering");
        
        commands.insert_resource(MatrixResource {
            matrix,
            offscreen_canvas: Some(offscreen_canvas),  // Wrap in Option for take() pattern
            last_valid_frame: None,  // Will be populated on first valid frame
        });
        println!("LED matrix initialized successfully with double buffering!");
    }

    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    {
        println!("Matrix feature not enabled - would initialize LED matrix");
    }
}

/// Setup the render target (offscreen texture that camera will render to)
/// Runs in PostStartup to ensure RenderDevice is available
fn setup_render_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
    mut camera_query: Query<&mut Camera>,
) {
    println!("Setting up 64x64 render target for headless rendering...");
    
    let size = Extent3d {
        width: RENDER_WIDTH,
        height: RENDER_HEIGHT,
        ..Default::default()
    };

    // Create the texture that will be rendered to
    // Using Rgba8UnormSrgb which is 4 bytes per pixel (R, G, B, A)
    let mut render_target_image =
        Image::new_target_texture(size.width, size.height, TextureFormat::Rgba8UnormSrgb);
    // Enable COPY_SRC so we can copy from this texture to a buffer
    render_target_image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let render_target_handle = images.add(render_target_image);

    // Configure the camera to render to our texture instead of a window
    match camera_query.single_mut() {
        Ok(mut camera) => {
            camera.target = RenderTarget::Image(render_target_handle.clone().into());
            println!("Camera configured to render to offscreen texture");
        }
        Err(e) => {
            eprintln!("ERROR: Failed to find camera for render target configuration: {:?}", e);
            panic!("Cannot continue without camera");
        }
    }

    // Spawn the ImageCopier component that will handle GPU→CPU transfer
    commands.spawn(ImageCopier::new(
        render_target_handle,
        size,
        &render_device,
    ));
    
    println!("Render target initialized successfully");
}

/// Plugin that handles copying rendered frames from GPU to CPU
struct ImageCopyPlugin;

impl Plugin for ImageCopyPlugin {
    fn build(&self, app: &mut App) {
        // Create channel for GPU→CPU frame data transfer
        let (sender, receiver) = crossbeam_channel::unbounded();

        // Store receiver in main world
        app.insert_resource(MainWorldReceiver(receiver));

        // Access render sub-app and configure it
        let render_app = app.sub_app_mut(RenderApp);

        // Add our custom render graph node for copying texture→buffer
        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyDriver);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        // Store sender in render world and add systems
        render_app
            .insert_resource(RenderWorldSender(sender))
            // Extract ImageCopiers from main world to render world
            .add_systems(ExtractSchedule, image_copy_extract)
            // Receive image data from GPU buffer and send via channel
            // This runs after rendering is complete
            .add_systems(Render, receive_image_from_buffer);
    }
}

/// Component that tracks a render target and its associated CPU-readable buffer
#[derive(Clone, Component)]
struct ImageCopier {
    buffer: Buffer,
    enabled: Arc<AtomicBool>,
    src_image: Handle<Image>,
    pub padded_bytes_per_row: usize,
}

impl ImageCopier {
    pub fn new(
        src_image: Handle<Image>,
        size: Extent3d,
        render_device: &RenderDevice,
    ) -> ImageCopier {
        // Calculate row padding for GPU alignment requirements
        // IMPORTANT: Must match the calculation in ImageCopyDriver::run()
        // For RGBA8: width * 4 bytes per pixel, then align
        let unpadded_bytes_per_row = size.width as usize * 4; // RGBA = 4 bytes per pixel
        let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(unpadded_bytes_per_row);

        // Create CPU-readable buffer for frame data
        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("matrix_frame_buffer"),
            size: padded_bytes_per_row as u64 * size.height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        ImageCopier {
            buffer: cpu_buffer,
            src_image,
            enabled: Arc::new(AtomicBool::new(true)),
            padded_bytes_per_row,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

/// Resource to hold all ImageCopiers in render world
#[derive(Clone, Default, Resource, Deref, DerefMut)]
struct ImageCopiers(Vec<ImageCopier>);

/// Extract ImageCopiers from main world into render world
fn image_copy_extract(mut commands: Commands, image_copiers: Extract<Query<&ImageCopier>>) {
    commands.insert_resource(ImageCopiers(
        image_copiers.iter().cloned().collect::<Vec<ImageCopier>>(),
    ));
}

/// Label for our custom render graph node
#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
struct ImageCopy;

/// Render graph node that copies texture to buffer
#[derive(Default)]
struct ImageCopyDriver;

impl render_graph::Node for ImageCopyDriver {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let image_copiers = world.get_resource::<ImageCopiers>().unwrap();
        let gpu_images = world
            .get_resource::<RenderAssets<bevy::render::texture::GpuImage>>()
            .unwrap();

        for image_copier in image_copiers.iter() {
            if !image_copier.enabled() {
                continue;
            }

            let src_image = gpu_images.get(&image_copier.src_image).unwrap();

            let mut encoder = render_context
                .render_device()
                .create_command_encoder(&CommandEncoderDescriptor::default());

            let block_dimensions = src_image.texture_format.block_dimensions();
            let block_size = src_image.texture_format.block_copy_size(None).unwrap();

            let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
                (src_image.size.width as usize / block_dimensions.0 as usize) * block_size as usize,
            );

            // Copy texture to buffer
            encoder.copy_texture_to_buffer(
                src_image.texture.as_image_copy(),
                TexelCopyBufferInfo {
                    buffer: &image_copier.buffer,
                    layout: TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZero::<u32>::new(padded_bytes_per_row as u32)
                                .unwrap()
                                .into(),
                        ),
                        rows_per_image: None,
                    },
                },
                src_image.size,
            );

            let render_queue = world.get_resource::<RenderQueue>().unwrap();
            render_queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }
}

/// System that receives frame data from GPU buffer and sends it to main world via channel
fn receive_image_from_buffer(
    image_copiers: Option<Res<ImageCopiers>>,
    render_device: Res<RenderDevice>,
    sender: Res<RenderWorldSender>,
) {
    // Check if ImageCopiers resource exists
    let Some(image_copiers) = image_copiers else {
        return; // Skip if not available yet
    };
    
    for image_copier in image_copiers.0.iter() {
        if !image_copier.enabled() {
            continue;
        }

        let buffer_slice = image_copier.buffer.slice(..);

        // Async callback for when buffer is mapped
        let (s, r) = crossbeam_channel::bounded(1);
        buffer_slice.map_async(MapMode::Read, move |result| match result {
            Ok(_) => s.send(()).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer: {err}"),
        });

        // Wait for GPU to finish
        render_device
            .poll(PollType::Wait)
            .expect("Failed to poll device");

        // Wait for buffer to be mapped
        r.recv().expect("Failed to receive map_async message");

        // Send buffer data to main world
        let _ = sender.send(buffer_slice.get_mapped_range().to_vec());

        // Unmap buffer for next frame
        image_copier.buffer.unmap();
    }
}

/// Check if frame data is valid (not all black/empty)
/// Returns true if at least some pixels have non-zero values
fn is_valid_frame(data: &[u8]) -> bool {
    // Sample every 256th byte to check quickly without iterating entire buffer
    // For a 64x64 RGBA image (16384 bytes), this checks ~64 samples
    data.iter().step_by(256).any(|&b| b > 0)
}

/// System in main world that receives frame data and displays it on LED matrix
/// CRITICAL: This must swap EVERY frame to maintain proper matrix library state
fn receive_and_display_frame(
    receiver: Res<MainWorldReceiver>,
    image_copiers: Query<&ImageCopier>,
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    mut matrix_res: ResMut<MatrixResource>,
) {
    let width = RENDER_WIDTH as usize;
    let height = RENDER_HEIGHT as usize;
    let row_bytes = width * 4; // RGBA = 4 bytes per pixel
    
    // Get the ImageCopier to know about row padding
    let Ok(image_copier) = image_copiers.single() else {
        // ImageCopier not ready yet - still swap to maintain timing
        #[cfg(all(target_os = "linux", feature = "matrix"))]
        {
            if let Some(canvas) = matrix_res.offscreen_canvas.take() {
                matrix_res.offscreen_canvas = Some(matrix_res.matrix.swap(canvas));
            }
        }
        return;
    };
    
    let aligned_row_bytes = image_copier.padded_bytes_per_row;
    
    // Try to get frame(s) from channel - use latest available
    let mut new_padded_data: Option<Vec<u8>> = None;
    while let Ok(data) = receiver.try_recv() {
        new_padded_data = Some(data);
    }
    
    // Process new frame if we got one
    let new_frame: Option<Vec<u8>> = new_padded_data.map(|padded_image_data| {
        // Debug output (only print once)
        static mut DEBUG_PRINTED: bool = false;
        unsafe {
            if !DEBUG_PRINTED {
                println!("DEBUG: Buffer size: {} bytes", padded_image_data.len());
                println!("DEBUG: Expected unpadded: {} bytes ({}x{} RGBA)", width * height * 4, width, height);
                println!("DEBUG: Row bytes: {}, Aligned row bytes: {}", row_bytes, aligned_row_bytes);
                println!("DEBUG: Padding needed: {}", row_bytes != aligned_row_bytes);
                println!("DEBUG: Using DOUBLE BUFFERING + FRAME CACHING for flicker-free display");
                DEBUG_PRINTED = true;
            }
        }
        
        // Unpad the buffer if necessary (GPU buffers are aligned to 256 bytes per row)
        if row_bytes == aligned_row_bytes {
            padded_image_data
        } else {
            padded_image_data
                .chunks(aligned_row_bytes)
                .take(height)
                .flat_map(|row| &row[..row_bytes.min(row.len())])
                .cloned()
                .collect()
        }
    });
    
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    {
        use rpi_led_matrix::LedColor;
        
        // Track frame statistics for debugging
        static mut FRAME_NUM: u32 = 0;
        static mut BLACK_FRAME_COUNT: u32 = 0;
        static mut NO_FRAME_COUNT: u32 = 0;
        static mut VALID_FRAME_COUNT: u32 = 0;
        static mut NO_DISPLAY_COUNT: u32 = 0;
        
        unsafe { FRAME_NUM += 1; }
        
        // Determine which frame data to display:
        // 1. If we have a new valid frame, use it and cache it
        // 2. If new frame is black/invalid, use cached frame
        // 3. If no new frame, use cached frame
        // 4. If no cached frame either, swap with current canvas (maintains timing)
        
        // Clone the frame data to avoid borrow conflicts with canvas operations
        let (frame_to_display, frame_source): (Option<Vec<u8>>, &str) = if let Some(new_data) = new_frame {
            if is_valid_frame(&new_data) {
                // Valid new frame - cache it and use it
                unsafe { VALID_FRAME_COUNT += 1; }
                matrix_res.last_valid_frame = Some(new_data.clone());
                (Some(new_data), "new_valid")
            } else {
                // New frame is black/invalid - use cached if available
                unsafe { BLACK_FRAME_COUNT += 1; }
                (matrix_res.last_valid_frame.clone(), "cached_after_black")
            }
        } else {
            // No new frame from GPU - use cached
            unsafe { NO_FRAME_COUNT += 1; }
            (matrix_res.last_valid_frame.clone(), "cached_no_new")
        };
        
        let has_cached = matrix_res.last_valid_frame.is_some();
        let will_draw = frame_to_display.is_some();
        
        // Log every frame for first 10, then every 10th frame
        unsafe {
            if FRAME_NUM <= 10 || FRAME_NUM % 10 == 0 {
                println!("FRAME {}: source={}, has_cached={}, will_draw={} (valid={}, black={}, no_new={}, no_draw={})",
                    FRAME_NUM, frame_source, has_cached, will_draw,
                    VALID_FRAME_COUNT, BLACK_FRAME_COUNT, NO_FRAME_COUNT, NO_DISPLAY_COUNT);
            }
        }
        
        // ALWAYS swap every frame - this is critical for matrix library timing
        if let Some(mut canvas) = matrix_res.offscreen_canvas.take() {
            if TEST_PATTERN_MODE {
                // TEST MODE: Draw a simple static pattern to isolate flicker source
                // This bypasses all GPU data - if this still flickers, issue is matrix library
                // Pattern: red square on black background (similar to working demo)
                let black = LedColor { red: 0, green: 0, blue: 0 };
                let red = LedColor { red: 100, green: 0, blue: 0 };
                
                // Fill with black
                for y in 0..height as i32 {
                    for x in 0..width as i32 {
                        canvas.set(x, y, &black);
                    }
                }
                
                // Draw 16x16 red square in center
                let sq_size = 16;
                let sq_start = (64 - sq_size) / 2;
                for y in sq_start..(sq_start + sq_size) {
                    for x in sq_start..(sq_start + sq_size) {
                        canvas.set(x, y, &red);
                    }
                }
                
                unsafe {
                    if FRAME_NUM <= 3 {
                        println!("TEST PATTERN MODE: Drawing static red square (bypassing GPU)");
                    }
                }
            } else if let Some(ref image_data) = frame_to_display {
                let expected_size = width * height * 4;
                if image_data.len() != expected_size {
                    eprintln!("WARNING: Image data size mismatch! Expected {}, got {}", 
                             expected_size, image_data.len());
                }
                
                // Draw every pixel to the OFFSCREEN canvas
                for y in 0..height {
                    for x in 0..width {
                        let pixel_idx = (y * width + x) * 4;
                        
                        if pixel_idx + 3 <= image_data.len() {
                            let r = image_data[pixel_idx];
                            let g = image_data[pixel_idx + 1];
                            let b = image_data[pixel_idx + 2];
                            
                            let color = LedColor { red: r, green: g, blue: b };
                            canvas.set(x as i32, y as i32, &color);
                        }
                    }
                }
            } else {
                // NO frame to display - this is bad, we'll show stale buffer content!
                unsafe { NO_DISPLAY_COUNT += 1; }
                // Fill with a debug color (dim red) to make this visible
                let debug_color = LedColor { red: 20, green: 0, blue: 0 };
                for y in 0..height as i32 {
                    for x in 0..width as i32 {
                        canvas.set(x, y, &debug_color);
                    }
                }
                println!("WARNING FRAME {}: No frame to display, filling with debug color!", unsafe { FRAME_NUM });
            }
            
            // ATOMIC SWAP: Always swap to maintain matrix library timing
            matrix_res.offscreen_canvas = Some(matrix_res.matrix.swap(canvas));
        }
    }
    
    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    {
        // Debug output for non-matrix builds
        if let Some(ref data) = new_frame {
            static mut FRAME_COUNTER: u32 = 0;
            unsafe {
                FRAME_COUNTER += 1;
                let is_valid = is_valid_frame(data);
                if FRAME_COUNTER % 30 == 1 {
                    println!("Frame {} received ({} bytes, valid: {})", FRAME_COUNTER, data.len(), is_valid);
                }
            }
        }
    }
}

