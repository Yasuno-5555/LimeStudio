// Lime Surface SDF Shaders
// Signed Distance Fields for perfect anti-aliasing and pseudo-glow.

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) shape_type: u32,
    @location(3) modulation_depth: f32,
    @location(4) modulation_current: f32,
    @location(5) params: vec4<f32>,
    @location(6) params2: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) shape_type: u32,
    @location(4) modulation_depth: f32,
    @location(5) modulation_current: f32,
    @location(6) params: vec4<f32>,
    @location(7) params2: vec4<f32>,
) -> VertexOutput {
    var pos = vec2<f32>(0.0, 0.0);
    var uv = vec2<f32>(0.0, 0.0);
    
    // Simple quad generation
    if (in_vertex_index == 0u) { pos = vec2<f32>(-1.0, 1.0); uv = vec2<f32>(-1.0, 1.0); }
    else if (in_vertex_index == 1u) { pos = vec2<f32>(-1.0, -1.0); uv = vec2<f32>(-1.0, -1.0); }
    else if (in_vertex_index == 2u) { pos = vec2<f32>(1.0, 1.0); uv = vec2<f32>(1.0, 1.0); }
    else if (in_vertex_index == 3u) { pos = vec2<f32>(1.0, -1.0); uv = vec2<f32>(1.0, -1.0); }

    var out: VertexOutput;
    // Map screen space to clip space (Simplified, assuming 1:1 for now)
    out.clip_position = vec4<f32>(position + pos * size, 0.0, 1.0);
    out.uv = uv;
    out.color = color;
    out.shape_type = shape_type;
    out.modulation_depth = modulation_depth;
    out.modulation_current = modulation_current;
    out.params = params;
    out.params2 = params2;
    return out;
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

// Quadratic Bezier SDF (Approximation for performance)
fn sd_quadratic_bezier(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
    let ba = b - a;
    let cb = c - b;
    let pa = p - a;
    
    // We approximate with 4 segments for better performance than solving cubic roots
    let p0 = a;
    let p1 = mix(mix(a, b, 0.33), mix(b, c, 0.33), 0.33);
    let p2 = mix(mix(a, b, 0.66), mix(b, c, 0.66), 0.66);
    let p3 = c;
    
    var d = sd_segment(p, p0, p1);
    d = min(d, sd_segment(p, p1, p2));
    d = min(d, sd_segment(p, p2, p3));
    return d;
}

// Perceptual mix (placeholder for Oklab implementation in WGSL)
fn mix_perceptual(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    // For now, linear mix, but documentation says Oklab is mandatory.
    return mix(a, b, t);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var d = 1.0;
    
    if (in.shape_type == 0u) {
        // Circle
        d = sd_circle(in.uv, 1.0);
    } else if (in.shape_type == 1u) {
        // Rounded Box
        d = sd_rounded_box(in.uv, vec2<f32>(1.0), in.params);
    } else if (in.shape_type == 2u) {
        // Arc (params: [sc.x, sc.y, ra, rb])
        d = sd_arc(in.uv, in.params.xy, in.params.z, in.params.w);
    } else if (in.shape_type == 3u) {
        // Modulation Ring (The Logic of Signal Visibility)
        // params: [inner_r, outer_r, base_angle, range_angle]
        let r = length(in.uv);
        let inner = in.params.x;
        let outer = in.params.y;
        let base_angle = in.params.z;
        let range = in.params.w; // depth
        
        // Ring mask
        let ring_d = abs(r - (inner + outer) * 0.5) - (outer - inner) * 0.5;
        
        // Angle calculation
        let angle = atan2(in.uv.x, in.uv.y); // -PI to PI
        
        // 1. Base Modulation Range Arc
        let mod_min = base_angle - range;
        let mod_max = base_angle + range;
        
        // Circular distance to range
        var angle_dist = 1.0;
        if (angle >= mod_min && angle <= mod_max) {
            angle_dist = -1.0;
        }
        
        // 2. Current Position Indicator
        let current_angle = base_angle + in.modulation_current * range;
        let dist_to_current = abs(angle - current_angle);
        let indicator_d = dist_to_current - 0.05; // thin line
        
        d = max(ring_d, angle_dist);
        d = min(d, max(ring_d, indicator_d));
    } else if (in.shape_type == 4u) {
        // Glass Block (Authority Layer Background)
        // params: [half_width, half_height, corner_radius, _]
        d = sd_rounded_box(in.uv, in.params.xy, vec4<f32>(in.params.z));
    } else if (in.shape_type == 5u) {
        // PD_FRAME (Pd-style frames)
        // params: [style_type, stroke_width, _, _]
        let style = u32(in.params.x);
        let stroke = in.params.y;
        
        if (style == 1u) {
            // Message Box Flag (Right side extension)
            let q = in.uv;
            let d1 = sd_rounded_box(q, vec2<f32>(1.0, 1.0), vec4<f32>(0.0));
            // Flag cut (Simplified: just a rectangle with a bite taken out or a flag tail)
            // Pure Data message box has a small 'tail' on the right.
            let tail = sd_segment(q, vec2<f32>(1.0, -0.5), vec2<f32>(1.2, 0.0));
            d = min(d1, tail - 0.05);
        } else if (style == 2u) {
            // Number Box (Chamfered top-left)
            d = sd_chamfered_box(in.uv, vec2<f32>(1.0, 1.0), 0.3);
        } else {
            // Standard Rect
            d = sd_rounded_box(in.uv, vec2<f32>(1.0, 1.0), vec4<f32>(0.0));
        }
        
        // Convert to stroke if needed
        if (stroke > 0.0) {
            d = abs(d) - stroke;
        }
    } else if (in.shape_type == 6u) {
        // PD_INDICATOR (Pd-style status indicators)
        // params: [kind, state, _, _]
        let kind = u32(in.params.x);
        let state = in.params.y;
        
        if (kind == 0u) {
            // Bang (Circle in Square)
            let outer = sd_rounded_box(in.uv, vec2<f32>(1.0), vec4<f32>(0.0));
            let inner = sd_circle(in.uv, 0.7 * state); // Dynamic radius based on flash state
            d = min(abs(outer) - 0.05, inner);
        } else if (kind == 1u) {
            // Toggle (X in Square)
            let outer = sd_rounded_box(in.uv, vec2<f32>(1.0), vec4<f32>(0.0));
            let x1 = sd_segment(in.uv, vec2<f32>(-0.7), vec2<f32>(0.7));
            let x2 = sd_segment(in.uv, vec2<f32>(0.7, -0.7), vec2<f32>(-0.7, 0.7));
            let cross = min(x1, x2) - 0.1 * state;
            d = min(abs(outer) - 0.05, cross);
        } else if (kind == 2u) {
            // Radio (Dot in Square)
            let outer = sd_rounded_box(in.uv, vec2<f32>(1.0), vec4<f32>(0.0));
            let inner = sd_circle(in.uv, 0.4 * state);
            d = min(abs(outer) - 0.05, inner);
        } else if (kind == 3u) {
            // Led (Solid Glow)
            d = sd_circle(in.uv, 0.8 * state);
        } else {
            d = sd_circle(in.uv, 0.5 * state);
        }
    } else if (in.shape_type == 7u) {
        // PD_TICKS (High-density repetitions)
        // params: [count, radius, angle_start, angle_end]
        let count = in.params.x;
        let radius = in.params.y;
        let start = in.params.z * 0.01745329251; // to rad
        let end = in.params.w * 0.01745329251;
        
        let r = length(in.uv);
        let angle = atan2(in.uv.x, in.uv.y);
        
        // 1. Distance to the ring
        let r_dist = abs(r - radius) - 0.05;
        
        // 2. Subdivide angle space
        let total_arc = end - start;
        let step = total_arc / (count - 1.0);
        
        // Circular distance to the nearest tick
        let normalized_angle = (angle - start + 3.14159) % 6.28318 - 3.14159;
        if (angle < start || angle > end) {
            d = 1.0;
        } else {
            let tick_idx = round((angle - start) / step);
            let tick_angle = start + tick_idx * step;
            let dist_to_tick = abs(angle - tick_angle) * r;
            d = max(r_dist, dist_to_tick - 0.01);
        }
    } else if (in.shape_type == 9u) {
        // PD_CURVE (Quadratic Bezier)
        let p = in.uv * 1.0; // Normalized -1..1
        // Convert to local space (in.size is half-extents)
        let local_p = in.uv * in.size;
        d = sd_quadratic_bezier(local_p, in.params.xy, in.params.zw, in.params2.xy) - in.params2.z;
    } else if (in.shape_type == 10u) {
        // CAUSALITY_LINK (Semantic Interaction Line)
        // params: [p0, p1], params2: [confidence, echo, _, _]
        // modulation_depth: importance, modulation_current: surprise
        let local_p = in.uv * in.size;
        let line_d = sd_segment(local_p, in.params.xy, in.params.zw);
        
        let importance = in.modulation_depth;
        let surprise = in.modulation_current;
        let confidence = in.params2.x;
        let echo = in.params2.y;
        
        // 1. Importance -> Thickness
        let thickness = 0.5 + importance * 1.5;
        let d = line_d - thickness;
        
        // 2. Certainty -> Sharpness (SDF edge width)
        let edge_width = 0.1 + (1.0 - confidence) * 2.0;
        
        // 3. Activity -> Consequence (Glow)
        let alpha = (1.0 - smoothstep(-edge_width, edge_width, d)) * echo;
        let glow = exp(-line_d * 0.1) * surprise * echo;
        
        return in.color * (alpha + glow * 0.5);
    } else if (in.shape_type == 11u) {
        // BAR (Meter/Status Bar)
        // params: [value, _, _, _]
        let val = in.params.x;
        let q = in.uv;
        // Bar is -1..1 in UV space.
        // If it's a vertical bar filling from bottom:
        let fill_line = 1.0 - (val * 2.0); // 1.0 is bottom, -1.0 is top
        d = sd_rounded_box(q, vec2<f32>(1.0, 1.0), vec4<f32>(0.0));
        
        if (q.y < fill_line) {
            d = 1.0; // Outside fill
        }
    }

    // Antialiasing using fwidth
    let fw = fwidth(d);
    let alpha = 1.0 - smoothstep(-fw, fw, d);
    
    // 1. Semantic Occlusion (排他の法 - Negative Space)
    // If we have an occlusion parameter, we 'eat' into the background.
    // params2.z = occlusion_margin
    let occlusion = in.params2.z;
    if (occlusion > 0.0) {
        // This logic requires a global SDF buffer or a multi-pass.
        // For a single-pass instance, we can only occlude 'within' the primitive.
        // We will implement this as a specialized pass in the engine.
    }
    
    // 2. Boolean Intersection (交差の法)
    // If the pixel is inside both A and B, flip lightness.

    // Handle Oklab-like mix for states (Simplified linear for now as per shader capability)
    let final_color = in.color;
    return vec4<f32>(final_color.rgb, final_color.a * alpha);
}
