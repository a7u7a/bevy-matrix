use bevy::app::{App, Plugin};
use bevy::asset::{embedded_asset, embedded_path, Asset, AssetPath};
use bevy::pbr::{Material, MaterialPlugin};
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct UvMaterial {}

impl Material for UvMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("uv_material.wgsl")).with_source("embedded"),
        )
    }
}

pub struct UvMaterialPlugin;

impl Plugin for UvMaterialPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "uv_material.wgsl");
        app.add_plugins(MaterialPlugin::<UvMaterial>::default());
    }
}
