// Shared 3D scene setup for both window and matrix modes
// This module contains the platform-agnostic scene configuration that will render
// to a window on Mac and to the LED matrix on Pi

use crate::uv_material::{UvMaterial, UvMaterialPlugin};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::Assets;
use bevy::camera::ClearColorConfig;
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::math::curve::{Curve, EaseFunction, EasingCurve};
use bevy::math::Vec3;
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
        app.add_plugins(UvMaterialPlugin);
        app.add_systems(Startup, setup_3d_scene);
        app.add_systems(Update, (rotate_cube, animate_cube_scale));
    }
}

/// Setup 3D scene with rotating red cube, lighting, and camera
/// Camera is spawned here first, then render target assigned later in headless mode
fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<UvMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 2.0, 3.5))),
        MeshMaterial3d(materials.add(UvMaterial {})),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.0, 1.0, 1.0)),
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

const SCALE_PERIOD_SECS: f32 = 1.0;
const SCALE_X_MIN: f32 = 1.0;
const SCALE_X_MAX: f32 = 2.0;

/// Rotate the cube at 90 degrees per second on Y axis
fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingCube>>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 1.0);
    }
}

/// Oscillate local X scale between 1.0 and 2.0 over a 1-second ping-pong cycle
fn animate_cube_scale(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingCube>>) {
    let t = time.elapsed_secs() % SCALE_PERIOD_SECS;
    let linear = if t < 0.5 { t * 2.0 } else { 2.0 - t * 2.0 };
    let eased = EaseFunction::CubicInOut.sample_clamped(linear);
    let scale_x = EasingCurve::new(SCALE_X_MIN, SCALE_X_MAX, EaseFunction::Linear)
        .sample(eased)
        .unwrap_or(SCALE_X_MIN);

    for mut transform in &mut query {
        transform.scale = Vec3::new(scale_x, 1.0, 1.0);
    }
}
