use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;


#[derive(Clone, Copy, Debug)]
pub struct VertexData {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: LinearRgba,
}

impl VertexData {
    pub fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self {
            pos: a.pos.lerp(b.pos, t),
            normal: a.normal.lerp(b.normal, t).normalize(),
            uv: a.uv.lerp(b.uv, t),
            color: LinearRgba::from_vec4(a.color.to_vec4().lerp(b.color.to_vec4(), t)),
        }
    }
}

impl PartialEq for VertexData {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.normal == other.normal && self.uv == other.uv && self.color == other.color
    }
}

impl Eq for VertexData {}

impl std::hash::Hash for VertexData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pos.x.to_bits().hash(state);
        self.pos.y.to_bits().hash(state);
        self.pos.z.to_bits().hash(state);
        self.normal.x.to_bits().hash(state);
        self.normal.y.to_bits().hash(state);
        self.normal.z.to_bits().hash(state);
        self.uv.x.to_bits().hash(state);
        self.uv.y.to_bits().hash(state);
        self.color.red.to_bits().hash(state);
        self.color.green.to_bits().hash(state);
        self.color.blue.to_bits().hash(state);
        self.color.alpha.to_bits().hash(state);
    }
}


pub fn split_mesh_by_planes(
    mesh: &Mesh,
    top_y: f32,
    bottom_y: f32,
    show_caps: bool,
    rim_thickness: f32,
) -> super::SlicedParts {
    // Collect vertices and indices
    let positions = if let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        p
    } else {
        return super::SlicedParts { head: None, body: None, legs: None, contours: Vec::new() };
    };
    let normals_storage;
    let normals = if let Some(VertexAttributeValues::Float32x3(n)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        n
    } else {
        normals_storage = vec![[0.0, 1.0, 0.0]; positions.len()];
        &normals_storage
    };
    let uvs_storage;
    let uvs = if let Some(VertexAttributeValues::Float32x2(u)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        u
    } else {
        uvs_storage = vec![[0.0, 0.0]; positions.len()];
        &uvs_storage
    };
    let colors_storage;
    let colors = if let Some(VertexAttributeValues::Float32x4(c)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        c
    } else {
        colors_storage = vec![[1.0, 1.0, 1.0, 1.0]; positions.len()];
        &colors_storage
    };
    let indices = match mesh.indices() {
        Some(Indices::U16(i)) => i.iter().map(|&x| x as usize).collect::<Vec<usize>>(),
        Some(Indices::U32(i)) => i.iter().map(|&x| x as usize).collect::<Vec<usize>>(),
        None => (0..positions.len()).collect::<Vec<usize>>(),
    };

    let start_time = std::time::Instant::now();
    debug!("Slicer: Processing {} tris", indices.len() / 3);

    // Process triangles sequentially for stability (Rayon removed due to thread contention issues)
    let (mut head_tris, mut body_tris, mut legs_tris, mut top_segments, mut bot_segments) = 
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for tri_indices in indices.chunks(3) {
        if tri_indices.len() < 3 { continue; }
        
        let tri_verts = [
            VertexData { 
                pos: positions[tri_indices[0]].into(), 
                normal: normals[tri_indices[0]].into(), 
                uv: uvs[tri_indices[0]].into(),
                color: LinearRgba::from_f32_array(colors[tri_indices[0]]),
            },
            VertexData { 
                pos: positions[tri_indices[1]].into(), 
                normal: normals[tri_indices[1]].into(), 
                uv: uvs[tri_indices[1]].into(),
                color: LinearRgba::from_f32_array(colors[tri_indices[1]]),
            },
            VertexData { 
                pos: positions[tri_indices[2]].into(), 
                normal: normals[tri_indices[2]].into(), 
                uv: uvs[tri_indices[2]].into(),
                color: LinearRgba::from_f32_array(colors[tri_indices[2]]),
            },
        ];

        let min_y = tri_verts[0].pos.y.min(tri_verts[1].pos.y).min(tri_verts[2].pos.y);
        let max_y = tri_verts[0].pos.y.max(tri_verts[1].pos.y).max(tri_verts[2].pos.y);

        if min_y > top_y {
            head_tris.push(tri_verts);
        } else if max_y < bottom_y {
            legs_tris.push(tri_verts);
        } else if min_y > bottom_y && max_y < top_y {
            body_tris.push(tri_verts);
        } else {
            let (top_above, top_below, top_contour) = split_triangle(&tri_verts, top_y);
            head_tris.extend(top_above);
            if let Some(s) = top_contour { top_segments.push(s); }

            for tri in top_below {
                let (bot_above, bot_below, bot_contour) = split_triangle(&tri, bottom_y);
                body_tris.extend(bot_above);
                legs_tris.extend(bot_below);
                if let Some(s) = bot_contour { bot_segments.push(s); }
            }
        }
    }


    let split_time = start_time.elapsed();

    let cap_time = if show_caps {
        let cap_start = std::time::Instant::now();
        // Capping: Add cap triangles to parts
        head_tris.extend(super::capper::build_caps_from_segments(&top_segments, false, rim_thickness));
        body_tris.extend(super::capper::build_caps_from_segments(&top_segments, true, rim_thickness));
        body_tris.extend(super::capper::build_caps_from_segments(&bot_segments, false, rim_thickness));
        legs_tris.extend(super::capper::build_caps_from_segments(&bot_segments, true, rim_thickness));
        cap_start.elapsed()
    } else {
        std::time::Duration::ZERO
    };
    info!("Slicing Speed: Split={:?}, Cap={:?} (Total={:?})", split_time, cap_time, start_time.elapsed());


    super::SlicedParts {
        head: Some(build_mesh_from_tris(&head_tris)),
        body: Some(build_mesh_from_tris(&body_tris)),
        legs: Some(build_mesh_from_tris(&legs_tris)),
        contours: [top_segments, bot_segments].concat(),
    }
}

fn split_triangle(
    tri: &[VertexData; 3],
    y: f32,
) -> (Vec<[VertexData; 3]>, Vec<[VertexData; 3]>, Option<[Vec3; 2]>) {
    let mut above_nodes = Vec::new();
    let mut below_nodes = Vec::new();

    for i in 0..3 {
        if tri[i].pos.y > y { above_nodes.push(i); } 
        else { below_nodes.push(i); }
    }

    if above_nodes.len() == 3 { return (vec![*tri], vec![], None); }
    if below_nodes.len() == 3 { return (vec![], vec![*tri], None); }

    let mut above_tris = Vec::new();
    let mut below_tris = Vec::new();

    // The triangle is split. Find the "lone" vertex
    let (lone_is_above, lone_idx, other_indices) = if above_nodes.len() == 1 {
        (true, above_nodes[0], [below_nodes[0], below_nodes[1]])
    } else {
        (false, below_nodes[0], [above_nodes[0], above_nodes[1]])
    };

    // Ensure correct winding by checking if lone_idx is between others
    let (i1, i2) = if (lone_idx + 1) % 3 == other_indices[0] {
        (other_indices[0], other_indices[1])
    } else {
        (other_indices[1], other_indices[0])
    };

    let v_lone = tri[lone_idx];
    let v1 = tri[i1];
    let v2 = tri[i2];

    // Calculate intersection points
    let t1 = (y - v_lone.pos.y) / (v1.pos.y - v_lone.pos.y);
    let t2 = (y - v_lone.pos.y) / (v2.pos.y - v_lone.pos.y);

    let v_int1 = VertexData::interpolate(&v_lone, &v1, t1);
    let v_int2 = VertexData::interpolate(&v_lone, &v2, t2);

    if lone_is_above {
        above_tris.push([v_lone, v_int1, v_int2]);
        below_tris.push([v_int1, v1, v2]);
        below_tris.push([v_int1, v2, v_int2]);
    } else {
        below_tris.push([v_lone, v_int1, v_int2]);
        above_tris.push([v_int1, v1, v2]);
        above_tris.push([v_int1, v2, v_int2]);
    }

    (above_tris, below_tris, Some([v_int1.pos, v_int2.pos]))
}

fn build_mesh_from_tris(tris: &[[VertexData; 3]]) -> Mesh {
    let mut pos = Vec::with_capacity(tris.len() * 3);
    let mut norm = Vec::with_capacity(tris.len() * 3);
    let mut uv = Vec::with_capacity(tris.len() * 3);
    let mut col = Vec::with_capacity(tris.len() * 3);
    let mut idx = Vec::with_capacity(tris.len() * 3);

    for (i, tri) in tris.iter().enumerate() {
        for v in tri {
            pos.push(v.pos.to_array());
            norm.push(v.normal.to_array());
            uv.push(v.uv.to_array());
            col.push(v.color.to_f32_array());
        }
        let start = (i * 3) as u32;
        idx.push(start);
        idx.push(start + 1);
        idx.push(start + 2);
    }


    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(pos),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(norm),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(uv),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(col),
    );
    mesh.insert_indices(Indices::U32(idx));
    mesh
}
