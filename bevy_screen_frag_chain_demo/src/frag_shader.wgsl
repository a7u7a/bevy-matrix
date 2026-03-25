#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct FragShaderMaterial {
    time: f32,
};

@group(2) @binding(0) var<uniform> material: FragShaderMaterial;

// Scale UV for 128×32 (4:1): more detail across width
const SCALE_X: f32 = 6.0;
const SCALE_Y: f32 = 1.5;

fn permute(x: vec3<f32>) -> vec3<f32> {
    return ((x * 34.0 + 1.0) * x) % 289.0;
}

fn simplex(v: vec2<f32>) -> f32 {
    let C = vec4<f32>(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);

    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    let i1 = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), x0.x > x0.y);
    let x12 = x0.xyxy + C.xxzz - vec4<f32>(i1, 1.0, 1.0);

    i = i % 289.0;
    let p = permute(permute(i.y + vec3<f32>(0.0, i1.y, 1.0)) + i.x + vec3<f32>(0.0, i1.x, 1.0));

    var m = max(0.5 - vec3<f32>(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3<f32>(0.0));
    m = m * m * m * m;

    let x = 2.0 * fract(p * C.www) - 1.0;
    let h = abs(x) - 0.5;
    let a0 = x - floor(x + 0.5);

    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    let g = vec3<f32>(
        a0.x * x0.x  + h.x * x0.y,
        a0.y * x12.x + h.y * x12.y,
        a0.z * x12.z + h.z * x12.w,
    );

    return 0.5 + 0.5 * 130.0 * dot(m, g);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<f32>(mesh.uv.x * SCALE_X, mesh.uv.y * SCALE_Y) + material.time;
    let n = simplex(p) / 1.5;

    // Visual split at chain boundary (first vs second panel)
    let left_panel = mesh.uv.x < 0.5;
    let tint_r = select(0.15, 0.45, left_panel);
    let tint_b = select(0.45, 0.15, left_panel);
    let tint_g = 0.12;

    return vec4<f32>(n * tint_r + 0.05, n * tint_g + 0.02, n * tint_b + 0.08, 1.0);
}
