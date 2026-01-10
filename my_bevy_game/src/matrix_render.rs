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
use bevy::ecs::schedule::IntoScheduleConfigs;
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

// LED Matrix types (only available on Linux with matrix feature)
#[cfg(all(target_os = "linux", feature = "matrix"))]
use rpi_led_matrix::{LedCanvas, LedMatrix, LedMatrixOptions};

/// Resource to hold the LED matrix hardware
/// This is created during startup and used to display rendered frames
#[cfg(all(target_os = "linux", feature = "matrix"))]
#[derive(Resource)]
pub struct MatrixResource {
    pub matrix: LedMatrix,
    pub canvas: Option<LedCanvas>,
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
        options.set_pwm_lsb_nanoseconds(130);
        
        // Optional: adjust brightness (0-100)
        if let Err(e) = options.set_brightness(50) {
            eprintln!("Warning: Failed to set brightness: {}", e);
        }
        
        println!("Matrix configuration: 64x64, adafruit-hat-pwm mapping");
        
        let matrix = LedMatrix::new(Some(options), None)
            .expect("Failed to create LED matrix - check hardware connection and permissions");
        
        // Create the offscreen canvas
        let canvas = matrix.offscreen_canvas();
        
        commands.insert_resource(MatrixResource {
            matrix,
            canvas: Some(canvas),
        });
        println!("LED matrix initialized successfully!");
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
    padded_bytes_per_row: usize,
}

impl ImageCopier {
    pub fn new(
        src_image: Handle<Image>,
        size: Extent3d,
        render_device: &RenderDevice,
    ) -> ImageCopier {
        // Calculate row padding for GPU alignment requirements
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row((size.width) as usize) * 4;

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

/// System in main world that receives frame data and displays it on LED matrix
fn receive_and_display_frame(
    receiver: Res<MainWorldReceiver>,
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    mut matrix_res: ResMut<MatrixResource>,
) {
    // Try to receive frame data (non-blocking)
    if let Ok(image_data) = receiver.try_recv() {
        #[cfg(all(target_os = "linux", feature = "matrix"))]
        {
            use rpi_led_matrix::LedColor;
            
            // Take the canvas out temporarily
            if let Some(mut canvas) = matrix_res.canvas.take() {
                // Convert RGBA → RGB and write to LED matrix
                // image_data is RGBA format: [R, G, B, A, R, G, B, A, ...]
                // We need to extract RGB and write to 64x64 matrix
                
                let width = RENDER_WIDTH as usize;
                let height = RENDER_HEIGHT as usize;
                
                for y in 0..height {
                    for x in 0..width {
                        // Calculate index in RGBA buffer (4 bytes per pixel)
                        let pixel_idx = (y * width + x) * 4;
                        
                        if pixel_idx + 2 < image_data.len() {
                            let r = image_data[pixel_idx];
                            let g = image_data[pixel_idx + 1];
                            let b = image_data[pixel_idx + 2];
                            // Alpha channel (pixel_idx + 3) is ignored
                            
                            let color = LedColor { red: r, green: g, blue: b };
                            canvas.set(x as i32, y as i32, &color);
                        }
                    }
                }
                
                // Swap buffers to display the new frame
                // This is hardware double-buffering required by the LED matrix library
                matrix_res.canvas = Some(matrix_res.matrix.swap(canvas));
            }
        }
        
        #[cfg(not(all(target_os = "linux", feature = "matrix")))]
        {
            println!("Matrix not available - received {} bytes of frame data", image_data.len());
        }
    }
}

