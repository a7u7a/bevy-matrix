// Explicit imports - only what we need for 3D rendering
use bevy::app::{App, Plugin, Startup};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::system::{Commands, ResMut};
use bevy::math::Vec3;
use bevy::prelude::{Camera3d, Cuboid, Mesh, Mesh3d, MeshMaterial3d, PointLight, StandardMaterial};
use bevy::transform::components::Transform;

/// A minimal plugin demonstrating 3D scene rendering
/// This plugin sets up a simple scene with a cube, light, and camera
/// It will only be active in window mode (Mac/desktop)
pub struct RenderDemoPlugin;

impl Plugin for RenderDemoPlugin {
    fn build(&self, app: &mut App) {
        // Add startup system to set up the 3D scene
        app.add_systems(Startup, setup_scene);
    }
}

/// Set up a simple 3D scene with a cube, light, and camera
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Setting up 3D scene...");
    
    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    
    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    println!("3D scene initialized successfully!");
}

