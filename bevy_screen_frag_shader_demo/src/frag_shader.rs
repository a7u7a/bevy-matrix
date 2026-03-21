use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{embedded_asset, embedded_path, Asset, AssetPath, Assets, Handle};
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::{Camera2d, Rectangle};
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{Material2d, Material2dPlugin, MeshMaterial2d};
use bevy::prelude::Mesh2d;
use bevy::time::Time;
use bevy::transform::components::Transform;

pub const RENDER_WIDTH: u32 = 64;
pub const RENDER_HEIGHT: u32 = 64;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FragShaderMaterial {
    #[uniform(0)]
    pub time: f32,
}

impl Material2d for FragShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("frag_shader.wgsl")).with_source("embedded"),
        )
    }
}

pub struct FragShaderPlugin;

impl Plugin for FragShaderPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "frag_shader.wgsl");
        app.add_plugins(Material2dPlugin::<FragShaderMaterial>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_time);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<bevy::mesh::Mesh>>,
    mut materials: ResMut<Assets<FragShaderMaterial>>,
) {
    commands.spawn(Camera2d);

    let material_handle = materials.add(FragShaderMaterial { time: 0.0 });
    commands.insert_resource(FragShaderMaterialHandle(material_handle.clone()));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            RENDER_WIDTH as f32,
            RENDER_HEIGHT as f32,
        ))),
        MeshMaterial2d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

#[derive(bevy::prelude::Resource)]
struct FragShaderMaterialHandle(Handle<FragShaderMaterial>);

fn update_time(
    time: Res<Time>,
    handle: Res<FragShaderMaterialHandle>,
    mut materials: ResMut<Assets<FragShaderMaterial>>,
) {
    if let Some(material) = materials.get_mut(&handle.0) {
        material.time = time.elapsed_secs();
    }
}
