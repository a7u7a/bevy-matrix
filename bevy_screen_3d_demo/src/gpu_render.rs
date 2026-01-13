// Test 5: GPU rendering with 3D scene (rotating cube)
// Adds full 3D rendering to Test 4's proven working pipeline

use bevy::app::{App, Plugin, PostStartup, PostUpdate, Update};
use bevy::asset::{Assets, Handle};
use bevy::camera::RenderTarget;
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::ecs::world::World;
use bevy::image::Image;
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Camera, Camera3d, Cuboid, Deref, DerefMut, Mesh, Mesh3d, MeshMaterial3d, PointLight, Resource, Transform};
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
use bevy::time::Time;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

const RENDER_WIDTH: u32 = 64;
const RENDER_HEIGHT: u32 = 64;

// LED Matrix types
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

/// Pre-buffer to hold stable frame data
/// This ensures all canvas drawing operations read from a consistent frame
#[derive(Resource)]
struct FrameBuffer {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl FrameBuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0u8; width * height * 4], // RGBA
            width,
            height,
        }
    }
}

/// Marker component for the rotating cube
#[derive(Component)]
struct RotatingCube;

/// Main plugin for GPU rendering with 3D scene
pub struct GpuRenderPlugin;

/// Resource to track the render target image handle
#[derive(Resource)]
struct RenderTargetHandle(Handle<Image>);

impl Plugin for GpuRenderPlugin {
    fn build(&self, app: &mut App) {
        // Initialize frame buffer
        app.insert_resource(FrameBuffer::new(
            RENDER_WIDTH as usize,
            RENDER_HEIGHT as usize,
        ));

        app.add_systems(
            PostStartup,
            (initialize_matrix, setup_render_target, setup_3d_scene).chain(),
        );
        
        // Add rotation system that runs every frame
        app.add_systems(Update, rotate_cube);
        
        app.add_plugins(ImageCopyPlugin);
        app.add_systems(PostUpdate, receive_and_display_frame);
    }
}

/// Initialize the LED matrix hardware
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
        options.set_brightness(50);

        let matrix = LedMatrix::new(Some(options), None).expect("Failed to create LED matrix");

        let offscreen_canvas = matrix.offscreen_canvas();

        commands.insert_resource(MatrixResource {
            matrix,
            offscreen_canvas: Some(offscreen_canvas),
        });
        println!("LED matrix initialized!");
    }

    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    {
        println!("Matrix feature not enabled");
    }
}

/// Setup render target and 3D camera
fn setup_render_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    println!("Setting up 64x64 GPU render target with 3D camera...");

    let size = Extent3d {
        width: RENDER_WIDTH,
        height: RENDER_HEIGHT,
        ..Default::default()
    };

    // Create offscreen render target
    let mut render_target_image =
        Image::new_target_texture(size.width, size.height, TextureFormat::Rgba8UnormSrgb);
    render_target_image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let render_target_handle = images.add(render_target_image);

    // Store handle
    commands.insert_resource(RenderTargetHandle(render_target_handle.clone()));

    // Spawn a 3D camera rendering to our texture
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: RenderTarget::Image(render_target_handle.clone().into()),
            ..Default::default()
        },
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn the ImageCopier
    commands.spawn(ImageCopier::new(render_target_handle, size, &render_device));

    println!("GPU render target initialized - 3D camera ready");
}

/// Setup 3D scene with rotating red cube and lighting
fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Setting up 3D scene: red cube + point light...");

    // Red cube at origin with rotation marker
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 4.0, 4.0))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))), // Pure red
        Transform::from_xyz(0.0, 0.0, 0.0),
        RotatingCube, // Marker component for rotation system
    ));

    // Point light positioned above and to the side
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    println!("3D scene initialized!");
}

/// Rotate the cube at 90 degrees per second on Y axis
fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingCube>>) {
    for mut transform in &mut query {
        // Rotate 90 degrees per second = π/2 radians per second ≈ 1.57 rad/s
        transform.rotate_y(time.delta_secs() * 1.57);
    }
}

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

/// PRE-BUFFER STRATEGY: Copy GPU data to stable buffer first, then draw from buffer
/// This ensures matrix refresh thread sees consistent intermediate states
fn receive_and_display_frame(
    receiver: Res<MainWorldReceiver>,
    image_copiers: Query<&ImageCopier>,
    mut frame_buffer: ResMut<FrameBuffer>,
    #[cfg(all(target_os = "linux", feature = "matrix"))] mut matrix_res: ResMut<MatrixResource>,
) {
    static mut FRAME_COUNT: u32 = 0;

    let Ok(mut padded_image_data) = receiver.try_recv() else {
        return;
    };

    unsafe {
        FRAME_COUNT += 1;
    }

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

    // PHASE 1: Fast copy to pre-buffer (instant memcpy ~1µs)
    let copy_start = std::time::Instant::now();
    frame_buffer.data.copy_from_slice(&image_data);
    let copy_duration = copy_start.elapsed();

    #[cfg(all(target_os = "linux", feature = "matrix"))]
    {
        use rpi_led_matrix::LedColor;

        if let Some(mut canvas) = matrix_res.offscreen_canvas.take() {
            // PHASE 2: Draw from stable buffer to canvas
            let draw_start = std::time::Instant::now();

            for y in 0..height {
                for x in 0..width {
                    let pixel_idx = (y * width + x) * 4;

                    if pixel_idx + 3 <= frame_buffer.data.len() {
                        let r = frame_buffer.data[pixel_idx];
                        let g = frame_buffer.data[pixel_idx + 1];
                        let b = frame_buffer.data[pixel_idx + 2];

                        let color = LedColor {
                            red: r,
                            green: g,
                            blue: b,
                        };
                        canvas.set(x as i32, y as i32, &color);
                    }
                }
            }

            let draw_duration = draw_start.elapsed();

            // Swap canvas
            matrix_res.offscreen_canvas = Some(matrix_res.matrix.swap(canvas));

            unsafe {
                if FRAME_COUNT <= 10 || FRAME_COUNT % 60 == 0 {
                    println!(
                        "FRAME {}: copy={:?}, draw={:?}, total={:?}",
                        FRAME_COUNT,
                        copy_duration,
                        draw_duration,
                        copy_duration + draw_duration
                    );
                }
            }
        }
    }

    #[cfg(not(all(target_os = "linux", feature = "matrix")))]
    {
        unsafe {
            if FRAME_COUNT <= 10 || FRAME_COUNT % 60 == 0 {
                println!(
                    "FRAME {}: copy={:?} ({} bytes)",
                    FRAME_COUNT,
                    copy_duration,
                    image_data.len()
                );
            }
        }
    }
}

