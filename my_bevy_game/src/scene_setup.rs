// Shared 3D scene setup for both window and matrix modes
// This module contains the platform-agnostic scene configuration that will render
// to a window on Mac and to the LED matrix on Pi

use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::system::{Commands, ResMut};
use bevy::math::Vec3;
use bevy::prelude::{Camera3d, Cuboid, Mesh, Mesh3d, MeshMaterial3d, PointLight, StandardMaterial};
use bevy::transform::components::Transform;

/// Target resolution for rendering (matches LED matrix dimensions)
pub const RENDER_WIDTH: u32 = 64;
pub const RENDER_HEIGHT: u32 = 64;

/// Set up a simple 3D scene with a cube, light, and camera
/// This scene is shared between window mode (Mac) and matrix mode (Pi)
pub fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Setting up shared 3D scene (64x64)...");
    
    // Cube - positioned slightly above origin
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    
    // Point light - positioned above and to the side
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 1500.0,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    // Camera - positioned to view the cube
    // This camera will either render to a window or to a render target texture
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    println!("3D scene initialized: cube, light, and camera");
}

