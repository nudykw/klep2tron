use bevy::prelude::*;

pub fn build_caps_from_segments(segments: &[[Vec3; 2]]) -> Vec<[super::slicer::VertexData; 3]> {
    if segments.is_empty() { return Vec::new(); }

    let mut loops = Vec::new();
    let mut unused = segments.to_vec();

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
                if (s[0] - last).length_squared() < 0.00001 {
                    last = s[1];
                    unused.remove(i);
                    found = true;
                    break;
                } else if (s[1] - last).length_squared() < 0.00001 {
                    last = s[0];
                    unused.remove(i);
                    found = true;
                    break;
                }
            }
            if found { current_loop.push(last); }
        }
        if current_loop.len() >= 3 { loops.push(current_loop); }
    }

    let mut all_tris = Vec::new();
    for l in loops {
        all_tris.extend(triangulate_polygon(&l));
    }
    all_tris
}

pub fn triangulate_polygon(vertices: &[Vec3]) -> Vec<[super::slicer::VertexData; 3]> {
    let count = vertices.len();
    if count < 3 { return Vec::new(); }

    let mut tris = Vec::new();
    let center_normal = Vec3::Y; // Always Y-aligned

    for i in 2..count {
        let v0 = super::slicer::VertexData { pos: vertices[0], normal: center_normal, uv: [vertices[0].x, vertices[0].z].into() };
        let v1 = super::slicer::VertexData { pos: vertices[i-1], normal: center_normal, uv: [vertices[i-1].x, vertices[i-1].z].into() };
        let v2 = super::slicer::VertexData { pos: vertices[i], normal: center_normal, uv: [vertices[i].x, vertices[i].z].into() };
        tris.push([v0, v1, v2]);
    }
    tris
}
