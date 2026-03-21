#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct FragShaderMaterial {
    time: f32,
};

@group(2) @binding(0) var<uniform> material: FragShaderMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let t = material.time;
    return vec4<f32>(
        uv.x,
        uv.y,
        sin(t) * 0.5 + 0.5,
        1.0
    );
}
