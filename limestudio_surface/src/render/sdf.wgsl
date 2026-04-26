// Lime Surface SDF Shaders
// Signed Distance Fields for perfect anti-aliasing and pseudo-glow.

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// Circle SDF
fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

// Arc SDF for modulation rings
fn sd_arc(p: vec2<f32>, sc: vec2<f32>, ra: f32, rb: f32) -> f32 {
    var p_mut = p;
    p_mut.x = abs(p_mut.x);
    let k = if sc.y * p_mut.x > sc.x * p_mut.y { dot(p_mut, sc) } else { length(p_mut) };
    return sqrt(dot(p_mut, p_mut) + ra * ra - 2.0 * ra * k) - rb;
}

// Rounded Box SDF
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var r_val = r.x;
    if (p.x > 0.0 && p.y > 0.0) { r_val = r.x; }
    else if (p.x > 0.0 && p.y <= 0.0) { r_val = r.y; }
    else if (p.x <= 0.0 && p.y <= 0.0) { r_val = r.z; }
    else { r_val = r.w; }
    
    let q = abs(p) - b + r_val;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r_val;
}

// Chamfered Box SDF
fn sd_chamfered_box(p: vec2<f32>, b: vec2<f32>, c: f32) -> f32 {
    let q = abs(p) - b;
    let d1 = max(q.x, q.y);
    let d2 = (q.x + q.y + c) * 0.70710678118;
    return max(d1, d2);
}

// Line segment SDF
fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// Perceptual mix (placeholder for Oklab implementation in WGSL)
fn mix_perceptual(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    // For now, linear mix, but documentation says Oklab is mandatory.
    return mix(a, b, t);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // This is a generic fragment shader that would be specialized based on the shape.
    // In a real SDF renderer, we would pass shape parameters via uniforms.
    return in.color;
}
