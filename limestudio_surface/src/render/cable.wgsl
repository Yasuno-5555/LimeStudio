struct GlobalUniforms {
    view_proj: mat4x4<f32>,
    time: f32,
};
@group(0) @binding(0) var<uniform> globals: GlobalUniforms;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) start: vec2<f32>,
    @location(1) end: vec2<f32>,
    @location(2) cp1: vec2<f32>,
    @location(3) cp2: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) thickness: f32,
    @location(6) intensity: f32,
    @location(7) speed: f32,
    @location(8) phase: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) start: vec2<f32>,
    @location(2) end: vec2<f32>,
    @location(3) cp1: vec2<f32>,
    @location(4) cp2: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(6) thickness: f32,
    @location(7) intensity: f32,
    @location(8) speed: f32,
    @location(9) phase: f32,
    @location(10) progress: f32,
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
    
    // Better bounding box including control points
    let padding = input.thickness + 12.0; 
    let min_p = min(min(input.start, input.end), min(input.cp1, input.cp2)) - padding;
    let max_p = max(max(input.start, input.end), max(input.cp1, input.cp2)) + padding;
    
    let world_pos = min_p + (uv * 0.5 + 0.5) * (max_p - min_p);
    
    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.world_pos = world_pos;
    out.start = input.start;
    out.end = input.end;
    out.cp1 = input.cp1;
    out.cp2 = input.cp2;
    out.color = input.color;
    out.thickness = input.thickness;
    out.intensity = input.intensity;
    out.speed = input.speed;
    out.phase = input.phase;

    return out;
}


fn cubic_bezier(t: f32, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> vec2<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    return p0 * mt3 + p1 * (3.0 * mt2 * t) + p2 * (3.0 * mt * t2) + p3 * t3;
}

fn sdf_cubic_bezier(p: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> vec3<f32> {
    var min_d = 1e10;
    var min_t = 0.0;
    
    // 8-step search for closest t
    let steps = 8;
    for (var i = 0; i <= steps; i = i + 1) {
        let t = f32(i) / f32(steps);
        let pos = cubic_bezier(t, p0, p1, p2, p3);
        let d = length(p - pos);
        if (d < min_d) {
            min_d = d;
            min_t = t;
        }
    }
    
    // One subdivision step for better accuracy
    let step_size = 1.0 / f32(steps);
    let t_start = max(0.0, min_t - step_size);
    let t_end = min(1.0, min_t + step_size);
    for (var i = 1; i < 4; i = i + 1) {
        let t = mix(t_start, t_end, f32(i) / 4.0);
        let pos = cubic_bezier(t, p0, p1, p2, p3);
        let d = length(p - pos);
        if (d < min_d) {
            min_d = d;
            min_t = t;
        }
    }

    return vec3<f32>(min_d, min_t, 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let res = sdf_cubic_bezier(in.world_pos, in.start, in.cp1, in.cp2, in.end);
    let d = res.x;
    let progress = res.y;
    
    // Basic cable thickness
    let alpha = 1.0 - smoothstep(in.thickness * 0.5 - 1.0, in.thickness * 0.5 + 1.0, d);
    
    // Living Pulse (Temporal Truth)
    let p = fract(globals.time * in.speed + in.phase - progress);
    
    // Gaussian pulse shape
    let pulse_glow = exp(-pow(p - 0.5, 2.0) * 32.0) * in.intensity;
    
    // Dynamic thickness based on pulse
    let dynamic_thickness = in.thickness * (1.0 + pulse_glow * 0.5);
    let dynamic_alpha = 1.0 - smoothstep(dynamic_thickness * 0.5 - 1.0, dynamic_thickness * 0.5 + 1.0, d);
    
    if (dynamic_alpha <= 0.0) {
        discard;
    }
    
    let base_color = in.color.rgb;
    let glow_color = vec3<f32>(1.0, 1.0, 0.8); // Amber-ish glow
    let final_color = mix(base_color, glow_color, pulse_glow);
    
    return vec4<f32>(final_color, in.color.a * dynamic_alpha);
}

