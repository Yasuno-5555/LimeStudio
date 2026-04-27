struct GlobalUniforms {
    time: f32,
};
@group(0) @binding(0) var<uniform> globals: GlobalUniforms;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) start: vec2<f32>,
    @location(1) end: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) thickness: f32,
    @location(4) intensity: f32,
    @location(5) speed: f32,
    @location(6) phase: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) start: vec2<f32>,
    @location(2) end: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) thickness: f32,
    @location(5) intensity: f32,
    @location(6) speed: f32,
    @location(7) phase: f32,
    @location(8) @interpolate(linear) progress: f32,
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
    
    // Calculate a bounding box for the segment
    let padding = input.thickness + 5.0; // Add some room for glow
    let min_p = min(input.start, input.end) - padding;
    let max_p = max(input.start, input.end) + padding;
    let center = (min_p + max_p) * 0.5;
    let size = (max_p - min_p) * 0.5;
    
    let world_pos = center + (uv * size);
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(world_pos, 0.0, 1.0); // Placeholder NDC
    out.world_pos = world_pos;
    out.start = input.start;
    out.end = input.end;
    out.color = input.color;
    out.thickness = input.thickness;
    out.intensity = input.intensity;
    out.speed = input.speed;
    out.phase = input.phase;

    // Project world_pos onto the segment to find progress [0..1]
    let pa = world_pos - input.start;
    let ba = input.end - input.start;
    out.progress = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);

    return out;
}

fn sdf_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = sdf_segment(in.world_pos, in.start, in.end);
    
    // Basic cable thickness
    let alpha = 1.0 - smoothstep(in.thickness * 0.5 - 1.0, in.thickness * 0.5 + 1.0, d);
    
    // Living Pulse (Temporal Truth)
    // fract(time * speed + phase - progress) creates a wave moving along the cable
    let p = fract(globals.time * in.speed + in.phase - in.progress);
    
    // Gaussian pulse shape
    let pulse_glow = exp(-pow(p - 0.5, 2.0) * 32.0) * in.intensity;
    
    // Dynamic thickness based on pulse
    let dynamic_thickness = in.thickness * (1.0 + pulse_glow * 0.5);
    let dynamic_alpha = 1.0 - smoothstep(dynamic_thickness * 0.5 - 1.0, dynamic_thickness * 0.5 + 1.0, d);
    
    if (dynamic_alpha <= 0.0) {
        discard;
    }
    
    let base_color = in.color.rgb;
    let glow_color = vec3<f32>(1.0, 1.0, 0.8); // Amber-ish glow for pulses
    let final_color = mix(base_color, glow_color, pulse_glow);
    
    return vec4<f32>(final_color, in.color.a * dynamic_alpha);
}
