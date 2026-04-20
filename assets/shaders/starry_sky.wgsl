#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> sky_color: vec4<f32>;

fn hash(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash1(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453);
}

fn noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let f_sq = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(mix(hash(i + vec3(0.0, 0.0, 0.0)), hash(i + vec3(1.0, 0.0, 0.0)), f_sq.x),
            mix(hash(i + vec3(0.0, 1.0, 0.0)), hash(i + vec3(1.0, 1.0, 0.0)), f_sq.x), f_sq.y),
        mix(mix(hash(i + vec3(0.0, 0.0, 1.0)), hash(i + vec3(1.0, 0.0, 1.0)), f_sq.x),
            mix(hash(i + vec3(0.0, 1.0, 1.0)), hash(i + vec3(1.0, 1.0, 1.0)), f_sq.x), f_sq.y), f_sq.z);
}

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_position.xyz);
    var final_color = sky_color.rgb;
    
    // 1. Multi-layered Stars (Variable sizes and density)
    // Layer 1: Huge rare stars
    {
        let uv = dir * 25.0;
        let n = noise(uv);
        if (n > 0.98) {
            let h = hash(floor(uv));
            let twinkle = 0.5 + 0.5 * sin(globals.time * 2.0 + h * 50.0);
            let star = pow((n - 0.98) * 50.0, 1.8) * twinkle;
            final_color += vec3<f32>(star * 1.5);
        }
    }
    
    // Layer 2: Medium frequent stars
    {
        let uv = dir * 70.0;
        let n = noise(uv);
        if (n > 0.985) {
            let h = hash(floor(uv));
            let twinkle = 0.4 + 0.6 * sin(globals.time * 3.5 + h * 80.0);
            let star = pow((n - 0.985) * 66.0, 2.2) * twinkle;
            final_color += vec3<f32>(star * 1.0);
        }
    }
    
    // Layer 3: Small background stars (The "dust")
    {
        let uv = dir * 150.0;
        let n = noise(uv);
        if (n > 0.99) {
            let h = hash(floor(uv));
            let twinkle = 0.7 + 0.3 * sin(globals.time * 5.0 + h * 120.0);
            let star = pow((n - 0.99) * 100.0, 3.0) * twinkle;
            final_color += vec3<f32>(star * 0.4);
        }
    }

    // 2. Rare Meteors
    let time = globals.time;
    let meteor_freq = 5.0; 
    let cycle = floor(time / meteor_freq);
    let h_cycle = hash1(cycle);
    
    if (h_cycle > 0.7) { 
        let m_dir = normalize(vec3(hash1(cycle * 1.1) - 0.5, hash1(cycle * 1.2) - 0.5, hash1(cycle * 1.3) - 0.5));
        let m_tangent = normalize(cross(m_dir, vec3(0.0, 1.0, 0.0)));
        
        let local_time = fract(time / meteor_freq) * meteor_freq;
        let start_delay = hash1(cycle * 1.4) * (meteor_freq - 1.0);
        
        if (local_time > start_delay && local_time < start_delay + 0.6) {
            let progress = (local_time - start_delay) / 0.6;
            let streak_start = m_dir - m_tangent * 0.2;
            let streak_end = m_dir + m_tangent * 0.2;
            let current_pos = mix(streak_start, streak_end, progress);
            
            let dist_to_meteor = distance(dir, current_pos);
            let tail = exp(-distance(dir, mix(streak_start, current_pos, 0.5)) * 25.0) * (1.0 - progress);
            
            if (dist_to_meteor < 0.02) {
                let streak = (1.0 - dist_to_meteor * 50.0) * (1.0 - progress);
                final_color += vec3(streak * 2.5) + vec3(tail * 0.7);
            }
        }
    }
    
    return vec4<f32>(final_color, 1.0);
}
