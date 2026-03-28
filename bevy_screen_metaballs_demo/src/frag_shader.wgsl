// Metaballs — adapted from @patriciogv / Book of Shaders (GLSL → WGSL)

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct FragShaderMaterial {
    time: f32,
};

@group(2) @binding(0) var<uniform> material: FragShaderMaterial;

// Keep in sync with RENDER_WIDTH / RENDER_HEIGHT in frag_shader.rs
const RESOLUTION: vec2<f32> = vec2<f32>(64.0, 64.0);
const TAU: f32 = 6.283185307179586;

fn random2(p: vec2<f32>) -> vec2<f32> {
    return fract(
        sin(
            vec2<f32>(
                dot(p, vec2<f32>(127.1, 311.7)),
                dot(p, vec2<f32>(269.5, 183.3)),
            ),
        ) * 43758.5453,
    );
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var st = mesh.uv;
    st.x *= RESOLUTION.x / RESOLUTION.y;

    var color = vec3<f32>(0.0);

    // Scale
    st *= 10.0;

    // Tile the space
    let i_st = floor(st);
    let f_st = fract(st);

    var m_dist = 1.0;
    for (var j: i32 = -1; j <= 1; j++) {
        for (var i: i32 = -1; i <= 1; i++) {
            let neighbor = vec2<f32>(f32(i), f32(j));
            var offset = random2(i_st + neighbor);
            offset = 0.5 + 0.5 * sin(material.time + TAU * offset);
            let pos = neighbor + offset - f_st;
            let dist = length(pos);
            m_dist = min(m_dist, m_dist * dist);
        }
    }

    color += vec3<f32>(step(0.06, m_dist));

    return vec4<f32>(color, 1.0);
}
