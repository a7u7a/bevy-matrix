#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct FragShaderMaterial {
    time: f32,
};

@group(2) @binding(0) var<uniform> material: FragShaderMaterial;

const AMPLITUDE: f32 = 0.5;
const FREQUENCY: f32 = 3.0;

fn wave(p: f32, amplitude: f32, frequency: f32) -> f32 {
    return amplitude * abs(((p * frequency) % 2.0) - 1.0);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Remap UV from [0,1] to [-1,1], centered at origin
    let st = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);

    let d = length(st) - material.time;
    let w = wave(d, AMPLITUDE, FREQUENCY);

    return vec4<f32>(0, w, 0, 1.0);
}
