use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Clone, Copy, Debug)]
pub struct VertexData {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

impl VertexData {
    pub fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self {
            pos: a.pos.lerp(b.pos, t),
            normal: a.normal.lerp(b.normal, t).normalize(),
            uv: a.uv.lerp(b.uv, t),
        }
    }
}

pub fn split_mesh_by_planes(
    mesh: &Mesh,
    top_y: f32,
    bottom_y: f32,
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
    let indices = match mesh.indices() {
        Some(Indices::U16(i)) => i.iter().map(|&x| x as usize).collect::<Vec<usize>>(),
        Some(Indices::U32(i)) => i.iter().map(|&x| x as usize).collect::<Vec<usize>>(),
        None => (0..positions.len()).collect::<Vec<usize>>(),
    };

    debug!("Slicer: Processing {} tris", indices.len() / 3);

    let mut head_tris = Vec::new();
    let mut body_tris = Vec::new();
    let mut legs_tris = Vec::new();
    let mut top_segments = Vec::new();
    let mut bot_segments = Vec::new();

    for i in (0..indices.len()).step_by(3) {
        let tri_verts = [
            VertexData { pos: positions[indices[i]].into(), normal: normals[indices[i]].into(), uv: uvs[indices[i]].into() },
            VertexData { pos: positions[indices[i+1]].into(), normal: normals[indices[i+1]].into(), uv: uvs[indices[i+1]].into() },
            VertexData { pos: positions[indices[i+2]].into(), normal: normals[indices[i+2]].into(), uv: uvs[indices[i+2]].into() },
        ];

        // Split by Top Plane
        let (top_above, top_below, top_contour) = split_triangle(&tri_verts, top_y);
        for tri in top_above { head_tris.push(tri); }
        if let Some(segment) = top_contour { top_segments.push(segment); }

        // Split resulting 'below' by Bottom Plane
        for tri in top_below {
            let (bot_above, bot_below, bot_contour) = split_triangle(&tri, bottom_y);
            for b_tri in bot_above { body_tris.push(b_tri); }
            for b_tri in bot_below { legs_tris.push(b_tri); }
            if let Some(segment) = bot_contour { bot_segments.push(segment); }
        }
    }

    // Capping: Add cap triangles to parts
    head_tris.extend(super::capper::build_caps_from_segments(&top_segments, false));
    
    body_tris.extend(super::capper::build_caps_from_segments(&top_segments, true));
    body_tris.extend(super::capper::build_caps_from_segments(&bot_segments, false));

    legs_tris.extend(super::capper::build_caps_from_segments(&bot_segments, true));

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
    let mut pos = Vec::new();
    let mut norm = Vec::new();
    let mut uv = Vec::new();
    let mut idx = Vec::new();

    for (i, tri) in tris.iter().enumerate() {
        for v in tri {
            pos.push(v.pos.to_array());
            norm.push(v.normal.to_array());
            uv.push(v.uv.to_array());
        }
        idx.push((i * 3) as u32);
        idx.push((i * 3 + 1) as u32);
        idx.push((i * 3 + 2) as u32);
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
    mesh.insert_indices(Indices::U32(idx));
    mesh
}
