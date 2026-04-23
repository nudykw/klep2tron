use bevy::prelude::*;

/// Собирает треугольники для "заглушек" (caps) из набора сегментов.
pub fn build_caps_from_segments(segments: &[[Vec3; 2]], facing_up: bool) -> Vec<[super::slicer::VertexData; 3]> {
    if segments.is_empty() { return Vec::new(); }

    let mut loops = Vec::new();
    let mut unused = segments.to_vec();

    // Группируем сегменты в замкнутые петли
    while !unused.is_empty() {
        let mut current_loop = Vec::new();
        let start = unused.remove(0);
        current_loop.push(start[0]);
        let mut last = start[1];

        let mut found = true;
        while found {
            found = false;
            for i in 0..unused.len() {
                let s = unused[i];
                if (s[0] - last).length_squared() < 0.000001 {
                    last = s[1];
                    unused.remove(i);
                    found = true;
                    break;
                } else if (s[1] - last).length_squared() < 0.000001 {
                    last = s[0];
                    unused.remove(i);
                    found = true;
                    break;
                }
            }
            if found {
                current_loop.push(last);
            }
        }
        
        // Добавляем петлю, если в ней достаточно точек
        if current_loop.len() >= 3 {
            loops.push(current_loop);
        }
    }

    let mut all_tris = Vec::new();
    for l in loops {
        all_tris.extend(triangulate_polygon(&l, facing_up));
    }
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

    // Проверка на выпуклость угла (в 2D проекции XZ)
    let cross = (b.x - a.x) * (c.z - a.z) - (b.z - a.z) * (c.x - a.x);
    if cross <= 0.0 { return false; } // Угол вогнутый

    // Проверка, не лежат ли другие точки внутри этого треугольника
    for &idx in indices {
        if idx == p_idx || idx == c_idx || idx == n_idx { continue; }
        if point_in_triangle_2d(vertices[idx], a, b, c) {
            return false;
        }
    }

    true
}

fn point_in_triangle_2d(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
    let area_orig = ((a.x * (b.z - c.z) + b.x * (c.z - a.z) + c.x * (a.z - b.z)).abs()) / 2.0;
    let area1 = ((p.x * (a.z - b.z) + a.x * (b.z - p.z) + b.x * (p.z - a.z)).abs()) / 2.0;
    let area2 = ((p.x * (b.z - c.z) + b.x * (c.z - p.z) + c.x * (p.z - b.z)).abs()) / 2.0;
    let area3 = ((p.x * (c.z - a.z) + c.x * (a.z - p.z) + a.x * (p.z - c.z)).abs()) / 2.0;

    (area1 + area2 + area3 - area_orig).abs() < 0.0001
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
