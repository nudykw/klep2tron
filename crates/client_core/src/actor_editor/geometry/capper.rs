use bevy::prelude::*;
use std::collections::HashMap;
use rayon::prelude::*;



fn quantize(v: Vec3) -> [i32; 3] {
    [(v.x * 10000.0) as i32, (v.y * 10000.0) as i32, (v.z * 10000.0) as i32]
}

/// Собирает треугольники для "заглушек" (caps) из набора сегментов.
pub fn build_caps_from_segments(segments: &[[Vec3; 2]], facing_up: bool) -> Vec<[super::slicer::VertexData; 3]> {
    if segments.is_empty() { return Vec::new(); }

    let mut loops = Vec::new();
    let mut pos_to_segments: HashMap<[i32; 3], Vec<usize>> = HashMap::new();
    
    for (i, s) in segments.iter().enumerate() {
        pos_to_segments.entry(quantize(s[0])).or_default().push(i);
        pos_to_segments.entry(quantize(s[1])).or_default().push(i);
    }

    let mut used = vec![false; segments.len()];

    for i in 0..segments.len() {
        if used[i] { continue; }
        
        let mut current_loop = Vec::new();
        let current_seg_idx = i;
        used[current_seg_idx] = true;
        
        let start_pt = segments[current_seg_idx][0];
        let mut last_pt = segments[current_seg_idx][1];
        current_loop.push(start_pt);
        current_loop.push(last_pt);

        let start_q = quantize(start_pt);

        let mut loop_timeout = 0;
        loop {
            loop_timeout += 1;
            if loop_timeout > segments.len() { 
                warn!("Capper: Loop timeout reached! Polygon might be corrupted.");
                break; 
            }
            
            let last_q = quantize(last_pt);
            let mut found_next = false;
            
            if let Some(candidates) = pos_to_segments.get(&last_q) {
                for &next_idx in candidates {
                    if !used[next_idx] {
                        used[next_idx] = true;
                        let next_seg = segments[next_idx];
                        last_pt = if quantize(next_seg[0]) == last_q { next_seg[1] } else { next_seg[0] };
                        
                        if quantize(last_pt) != start_q {
                            current_loop.push(last_pt);
                        }
                        found_next = true;
                        break;
                    }
                }
            }
            
            if !found_next || quantize(last_pt) == start_q { break; }
        }


        if current_loop.len() >= 3 {
            loops.push(current_loop);
        }
    }


    let all_tris: Vec<_> = loops.par_iter()
        .map(|l| triangulate_polygon(l, facing_up))
        .flatten()
        .collect();
    
    all_tris
}


/// Триангуляция многоугольника методом Ear Clipping.
pub fn triangulate_polygon(vertices: &[Vec3], facing_up: bool) -> Vec<[super::slicer::VertexData; 3]> {
    let count = vertices.len();
    if count < 3 { return Vec::new(); }

    let mut tris = Vec::new();
    let normal = if facing_up { Vec3::Y } else { Vec3::NEG_Y };

    // Работаем с индексами для удобства удаления "ушей"
    let mut indices: Vec<usize> = (0..count).collect();

    // Определяем текущее направление обхода (поддерживаем CCW)
    let area = calculate_area_2d(vertices, &indices);
    let is_ccw = area > 0.0;
    
    // Если нам нужно смотреть вверх, а обход по часовой - инвертируем или наоборот
    // Для Bevy/Vulkan CCW - лицевая сторона.
    // Если facing_up=true, нам нужно, чтобы в итоге треугольники были CCW сверху.
    // Если facing_up=false, нам нужно, чтобы они были CCW снизу (т.е. CW сверху).
    let target_ccw = facing_up;
    if is_ccw != target_ccw {
        indices.reverse();
    }

    let mut timeout = 0;
    let max_timeout = count * 2;

    while indices.len() > 2 && timeout < max_timeout {
        let mut ear_found = false;
        for i in 0..indices.len() {
            let prev = indices[(i + indices.len() - 1) % indices.len()];
            let curr = indices[i];
            let next = indices[(i + 1) % indices.len()];

            if is_ear(prev, curr, next, &indices, vertices) {
                // Создаем треугольник
                let v0 = vertices[prev];
                let v1 = vertices[curr];
                let v2 = vertices[next];

                let vd0 = super::slicer::VertexData { pos: v0, normal, uv: Vec2::new(v0.x, v0.z) };
                let vd1 = super::slicer::VertexData { pos: v1, normal, uv: Vec2::new(v1.x, v1.z) };
                let vd2 = super::slicer::VertexData { pos: v2, normal, uv: Vec2::new(v2.x, v2.z) };

                tris.push([vd0, vd1, vd2]);
                
                indices.remove(i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            // Если ухо не найдено (сложный случай или самопересечение), 
            // пробуем принудительно отрезать любой треугольник, чтобы не зависнуть
            let i = 0;
            indices.remove(i);
        }
        timeout += 1;
    }

    tris
}

fn is_ear(p_idx: usize, c_idx: usize, n_idx: usize, indices: &[usize], vertices: &[Vec3]) -> bool {
    let a = vertices[p_idx];
    let b = vertices[c_idx];
    let c = vertices[n_idx];

    // 1. Convexity check (XZ projection)
    let cross = (b.x - a.x) * (c.z - a.z) - (b.z - a.z) * (c.x - a.x);
    if cross <= 0.0 { return false; } 

    // 2. AABB Culling (Fast path to skip 99% of points)
    let min_x = a.x.min(b.x).min(c.x);
    let max_x = a.x.max(b.x).max(c.x);
    let min_z = a.z.min(b.z).min(c.z);
    let max_z = a.z.max(b.z).max(c.z);

    // 3. Point-in-triangle check only for points inside AABB
    for &idx in indices {
        if idx == p_idx || idx == c_idx || idx == n_idx { continue; }
        let p = vertices[idx];
        
        if p.x >= min_x && p.x <= max_x && p.z >= min_z && p.z <= max_z {
            if point_in_triangle_2d_fast(p, a, b, c) {
                return false;
            }
        }
    }

    true
}

fn point_in_triangle_2d_fast(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
    let v0 = Vec2::new(c.x - a.x, c.z - a.z);
    let v1 = Vec2::new(b.x - a.x, b.z - a.z);
    let v2 = Vec2::new(p.x - a.x, p.z - a.z);

    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot02 = v0.dot(v2);
    let dot11 = v1.dot(v1);
    let dot12 = v1.dot(v2);

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    (u >= 0.0) && (v >= 0.0) && (u + v < 1.0)
}


fn calculate_area_2d(vertices: &[Vec3], indices: &[usize]) -> f32 {
    let mut area = 0.0;
    for i in 0..indices.len() {
        let j = (i + 1) % indices.len();
        let v1 = vertices[indices[i]];
        let v2 = vertices[indices[j]];
        area += (v1.x * v2.z) - (v2.x * v1.z);
    }
    area / 2.0
}
