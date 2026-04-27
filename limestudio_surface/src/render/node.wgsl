struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) corner_radius: f32,
    @location(4) border_thickness: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) size: vec2<f32>,
    @location(3) corner_radius: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, -1.0)
    );

    let uv = positions[input.vertex_index];
    let world_pos = input.pos + (uv * input.size * 0.5);
    
    // TODO: Apply Camera ViewProjection here. 
    // For now, assume world_pos is already in NDC for testing.
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(world_pos, 0.0, 1.0);
    out.uv = uv * input.size * 0.5;
    out.color = input.color;
    out.size = input.size;
    out.corner_radius = input.corner_radius;
    return out;
}

fn sdf_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = sdf_rounded_box(in.uv, in.size * 0.5, in.corner_radius);
    
    // Smooth edges
    let antialias = 1.0;
    let alpha = 1.0 - smoothstep(-antialias, antialias, d);
    
    if (alpha <= 0.0) {
        discard;
    }
    
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
