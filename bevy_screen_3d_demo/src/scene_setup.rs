// Shared 3D scene setup for both window and matrix modes
// This module contains the platform-agnostic scene configuration that will render
// to a window on Mac and to the LED matrix on Pi

use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::Assets;
use bevy::camera::ClearColorConfig;
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Camera, Camera3d, Cuboid, Mesh, Mesh3d, MeshMaterial3d, PointLight};
use bevy::time::Time;
use bevy::transform::components::Transform;
use matrix_render::MatrixCamera;

/// Marker component for the rotating cube
#[derive(Component)]
pub struct RotatingCube;

/// Plugin that sets up the shared 3D scene
/// Works on both window mode (Mac) and headless mode (Pi)
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_3d_scene);
        app.add_systems(Update, rotate_cube);
    }
}

/// Setup 3D scene with rotating red cube, lighting, and camera
/// Camera is spawned here first, then render target assigned later in headless mode
fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Red cube at origin with rotation marker
    let material = StandardMaterial {
        base_color: Color::srgb(0.0,1.0, 0.0), // Red
        ..Default::default()
    };

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 4.0, 4.0))),
        MeshMaterial3d(materials.add(material)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RotatingCube,
    ));

    // White point light positioned above and to the side
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            color: Color::WHITE,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..Default::default()
        },
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        MatrixCamera,
    ));
}

/// Rotate the cube at 90 degrees per second on Y axis
fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingCube>>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 0.57);
    }
}
