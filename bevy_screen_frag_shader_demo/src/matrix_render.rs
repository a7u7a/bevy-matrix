// Matrix rendering module for headless mode (Raspberry Pi)
// Handles GPU render target setup and LED matrix display

use bevy::app::{App, Plugin, PostStartup, PostUpdate};
use bevy::asset::{Assets, Handle};
use bevy::camera::RenderTarget;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::ecs::world::World;
use bevy::image::Image;
use bevy::prelude::{Camera, Camera2d, Deref, DerefMut, Resource};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{
    self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel,
};
use bevy::render::render_resource::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode,
    TexelCopyBufferInfo, TexelCopyBufferLayout, TextureFormat, TextureUsages,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp};
use crossbeam_channel::{Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};

use crate::frag_shader::{RENDER_HEIGHT, RENDER_WIDTH};

/// Frame counter for logging (using atomic to avoid static mut warnings)
static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);

// ============================================================================
// Ordered Dithering (8-bit → PWM_BITS quantization)
// ============================================================================

/// Must match the value passed to `options.set_pwm_bits()` in initialize_matrix
const PWM_BITS: u32 = 7;

/// Bayer 4x4 ordered dither threshold matrix.
/// Values 0..15 provide 16 spatially distributed thresholds.
#[rustfmt::skip]
const BAYER_4X4: [[u16; 4]; 4] = [
    [ 0,  8,  2, 10],
    [12,  4, 14,  6],
    [ 3, 11,  1,  9],
    [15,  7, 13,  5],
];

/// Dither a single 8-bit channel value to the available PWM output levels,
/// then map back to the 8-bit range the LED library expects.
#[inline]
fn dither_channel(value: u8, x: usize, y: usize) -> u8 {
    let max_output: u32 = (1 << PWM_BITS) - 1; // 127 for 7-bit
    let threshold = BAYER_4X4[y & 3][x & 3] as u32;

    // Scale input into output range with 4 extra fractional bits (×16)
    // so the Bayer threshold (0..15) can interpolate within one step.
    // u32 required: worst case 255 * 127 * 16 = 518,160 (overflows u16).
    let scaled = value as u32 * max_output * 16;
    let quantized = (scaled + threshold * 255) / (255 * 16);
    let clamped = quantized.min(max_output);

    // Map back to [0, 255] for the LED library
    (clamped * 255 / max_output) as u8
}

// ============================================================================
// LED Matrix Types
// ============================================================================
#[cfg(all(target_os = "linux", feature = "matrix"))]
use rpi_led_matrix::{LedCanvas, LedMatrix, LedMatrixOptions};

#[cfg(all(target_os = "linux", feature = "matrix"))]
#[derive(Resource)]
pub struct MatrixResource {
    pub matrix: LedMatrix,
    pub offscreen_canvas: Option<LedCanvas>,
}

#[cfg(all(target_os = "linux", feature = "matrix"))]
unsafe impl Send for MatrixResource {}
#[cfg(all(target_os = "linux", feature = "matrix"))]
unsafe impl Sync for MatrixResource {}

// ============================================================================
// Frame Buffer
// ============================================================================

/// Pre-buffer to hold stable frame data
/// Ensures all canvas drawing operations read from a consistent frame
#[derive(Resource)]
struct FrameBuffer {
    data: Vec<u8>,
}

impl FrameBuffer {
    fn new() -> Self {
        let size = RENDER_WIDTH as usize * RENDER_HEIGHT as usize * 4; // RGBA
        Self {
            data: vec![0u8; size],
        }
    }
}

/// Resource to track the render target image handle
#[derive(Resource)]
#[allow(dead_code)] // Handle is used via pattern matching in ImageCopier
struct RenderTargetHandle(Handle<Image>);

// ============================================================================
// Plugin
// ============================================================================

/// Plugin for GPU rendering to LED matrix in headless mode
pub struct MatrixRenderPlugin;

impl Plugin for MatrixRenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameBuffer::new());

        // PostStartup: Configure render target and matrix
        // Runs AFTER ScenePlugin's Startup, so the camera exists to be queried
        app.add_systems(
            PostStartup,
            (initialize_matrix, setup_render_target).chain(),
        );

        app.add_plugins(ImageCopyPlugin);
        app.add_systems(PostUpdate, receive_and_display_frame);
    }
}

// ============================================================================
// Matrix Initialization
// ============================================================================

/// Initialize the LED matrix hardware
fn initialize_matrix(mut commands: Commands) {
    #[cfg(all(target_os = "linux", feature = "matrix"))]
    {
        println!("Initializing LED matrix (64x64)...");

        let mut options = LedMatrixOptions::new();
        options.set_rows(64);
        options.set_cols(64);
        options.set_hardware_mapping("adafruit-hat-pwm");
        options.set_refresh_rate(true);

        // Display quality settings
        let _ = options.set_pwm_lsb_nanoseconds(300);
        let _ = options.set_pwm_bits(7);
        let _ = options.set_brightness(30);
        // let _ = options.set_pwm_dither_bits(1);

        let matrix = LedMatrix::new(Some(options), None).expect("Failed to create LED matrix");
        let offscreen_canvas = matrix.offscreen_canvas();

        commands.insert_resource(MatrixResource {
            matrix,
            offscreen_canvas: Some(offscreen_canvas),
        });
        println!("LED matrix initialized");
    }

    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    {
        // No-op on non-matrix platforms
    }
}

// ============================================================================
// Render Target Setup
// ============================================================================

/// Setup render target and configure the existing camera to render to it
fn setup_render_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
    mut camera_query: Query<&mut Camera, With<Camera2d>>,
) {
    let size = Extent3d {
        width: RENDER_WIDTH,
        height: RENDER_HEIGHT,
        ..Default::default()
    };

    // Create offscreen render target
    let texture_format = TextureFormat::Rgba8UnormSrgb;

    let mut render_target_image =
        Image::new_target_texture(size.width, size.height, texture_format);
    render_target_image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let render_target_handle = images.add(render_target_image);

    commands.insert_resource(RenderTargetHandle(render_target_handle.clone()));

    match camera_query.single_mut() {
        Ok(mut camera) => {
            camera.target = RenderTarget::Image(render_target_handle.clone().into());
        }
        Err(e) => {
            eprintln!("ERROR: Failed to find camera for render target: {:?}", e);
        }
    }

    // Spawn the ImageCopier
    commands.spawn(ImageCopier::new(render_target_handle, size, &render_device));
    println!(
        "GPU render target initialized ({}x{})",
        RENDER_WIDTH, RENDER_HEIGHT
    );
}

// ============================================================================
// Image Copy Pipeline (GPU -> CPU)
// ============================================================================

#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u8>>);

#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u8>>);

struct ImageCopyPlugin;

impl Plugin for ImageCopyPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        app.insert_resource(MainWorldReceiver(receiver));

        let render_app = app.sub_app_mut(RenderApp);
        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyDriver);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            .insert_resource(RenderWorldSender(sender))
            .add_systems(ExtractSchedule, image_copy_extract)
            .add_systems(Render, receive_image_from_buffer);
    }
}

#[derive(Clone, bevy::ecs::component::Component)]
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
        let unpadded_bytes_per_row = size.width as usize * 4;
        let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(unpadded_bytes_per_row);

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

#[derive(Clone, Default, Resource, Deref, DerefMut)]
struct ImageCopiers(Vec<ImageCopier>);

fn image_copy_extract(mut commands: Commands, image_copiers: Extract<Query<&ImageCopier>>) {
    commands.insert_resource(ImageCopiers(
        image_copiers.iter().cloned().collect::<Vec<ImageCopier>>(),
    ));
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
struct ImageCopy;

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

fn receive_image_from_buffer(
    image_copiers: Option<Res<ImageCopiers>>,
    render_device: Res<RenderDevice>,
    sender: Res<RenderWorldSender>,
) {
    let Some(image_copiers) = image_copiers else {
        return;
    };

    for image_copier in image_copiers.0.iter() {
        if !image_copier.enabled() {
            continue;
        }

        let buffer_slice = image_copier.buffer.slice(..);

        let (s, r) = crossbeam_channel::bounded(1);
        buffer_slice.map_async(MapMode::Read, move |result| match result {
            Ok(_) => s.send(()).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer: {err}"),
        });

        render_device
            .poll(bevy::render::render_resource::PollType::Wait)
            .expect("Failed to poll device");

        r.recv().expect("Failed to receive map_async message");

        let _ = sender.send(buffer_slice.get_mapped_range().to_vec());

        image_copier.buffer.unmap();
    }
}

// ============================================================================
// Frame Display
// ============================================================================

/// PRE-BUFFER STRATEGY: Copy GPU data to stable buffer first, then draw from buffer
/// This ensures matrix refresh thread sees consistent intermediate states
fn receive_and_display_frame(
    receiver: Res<MainWorldReceiver>,
    image_copiers: Query<&ImageCopier>,
    mut frame_buffer: ResMut<FrameBuffer>,
    #[cfg(all(target_os = "linux", feature = "matrix"))] mut matrix_res: ResMut<MatrixResource>,
) {
    let Ok(mut padded_image_data) = receiver.try_recv() else {
        return;
    };

    let frame_num = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

    // Get latest frame if multiple are queued
    while let Ok(newer_data) = receiver.try_recv() {
        padded_image_data = newer_data;
    }

    let Ok(image_copier) = image_copiers.single() else {
        return;
    };

    let width = RENDER_WIDTH as usize;
    let height = RENDER_HEIGHT as usize;
    let row_bytes = width * 4;
    let aligned_row_bytes = image_copier.padded_bytes_per_row;

    // Unpad if necessary
    let image_data: Vec<u8> = if row_bytes == aligned_row_bytes {
        padded_image_data
    } else {
        padded_image_data
            .chunks(aligned_row_bytes)
            .take(height)
            .flat_map(|row| &row[..row_bytes.min(row.len())])
            .cloned()
            .collect()
    };

    // Copy to pre-buffer
    frame_buffer.data.copy_from_slice(&image_data);

    #[cfg(all(target_os = "linux", feature = "matrix"))]
    {
        use rpi_led_matrix::LedColor;

        if let Some(mut canvas) = matrix_res.offscreen_canvas.take() {
            for y in 0..height {
                for x in 0..width {
                    let pixel_idx = (y * width + x) * 4;

                    if pixel_idx + 3 <= frame_buffer.data.len() {
                        let r = dither_channel(frame_buffer.data[pixel_idx], x, y);
                        let g = dither_channel(frame_buffer.data[pixel_idx + 1], x, y);
                        let b = dither_channel(frame_buffer.data[pixel_idx + 2], x, y);

                        canvas.set(
                            x as i32,
                            y as i32,
                            &LedColor {
                                red: r,
                                green: g,
                                blue: b,
                            },
                        );
                    }
                }
            }

            matrix_res.offscreen_canvas = Some(matrix_res.matrix.swap(canvas));
        }

        // For debugging
        if frame_num < 10 || frame_num % 300 == 0 {
            let center = (height / 2 * width + width / 2) * 4;
            let (cr, cg, cb) = (
                frame_buffer.data[center],
                frame_buffer.data[center + 1],
                frame_buffer.data[center + 2],
            );
            println!(
                "Frame {} displayed (center pixel: r={} g={} b={})",
                frame_num, cr, cg, cb
            );
        }
    }

    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    {
        // Periodic frame logging for non-matrix builds
        if frame_num < 10 || frame_num % 300 == 0 {
            println!("Frame {} processed ({} bytes)", frame_num, image_data.len());
        }
    }
}
