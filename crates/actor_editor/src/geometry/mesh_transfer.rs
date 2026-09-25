use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::{HashMap, HashSet};

/// Переносит выделенные треугольники из `src` в `dst`.
/// Возвращает (новый src без этих треугольников, новый dst с добавленными треугольниками).
/// Вершины дедуплицируются через vertex welding (HashMap по квантизованному ключу).
pub fn transfer_triangles(
    src: &Mesh,
    dst: &Mesh,
    triangles_to_move: &HashSet<usize>,
) -> (Mesh, Mesh) {
    let src_verts = extract_verts(src);
    let src_tris  = get_triangles(src);
    let dst_verts = extract_verts(dst);
    let dst_tris  = get_triangles(dst);

    // Разбиваем src на «остающиеся» и «переносимые»
    let mut keep_tris = Vec::new();
    let mut move_tris = Vec::new();
    for (i, tri) in src_tris.iter().enumerate() {
        if triangles_to_move.contains(&i) { move_tris.push(*tri); }
        else                              { keep_tris.push(*tri); }
    }

    // Новый src: только keep_tris, со сваркой вершин
    let new_src = build_welded_mesh(&src_verts, &keep_tris);

    // Новый dst: dst_tris + move_tris (вершины из src)
    // Объединяем два vertex-буфера и перестраиваем индексы move_tris с offset
    let combined_verts: Vec<_> = dst_verts.iter().chain(src_verts.iter()).cloned().collect();
    let offset = dst_verts.len();
    let mut all_tris = dst_tris.clone();
    for tri in &move_tris {
        all_tris.push([tri[0] + offset, tri[1] + offset, tri[2] + offset]);
    }
    let new_dst = build_welded_mesh(&combined_verts, &all_tris);

    (new_src, new_dst)
}

// ── Vertex welding ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Vert {
    pos: [f32; 3],
    nor: [f32; 3],
    uv:  [f32; 2],
}

fn quantize_vert(v: &Vert) -> [i32; 8] {
    [
        (v.pos[0] * 10000.0) as i32,
        (v.pos[1] * 10000.0) as i32,
        (v.pos[2] * 10000.0) as i32,
        (v.nor[0] * 10000.0) as i32,
        (v.nor[1] * 10000.0) as i32,
        (v.nor[2] * 10000.0) as i32,
        (v.uv[0]  * 10000.0) as i32,
        (v.uv[1]  * 10000.0) as i32,
    ]
}

fn build_welded_mesh(verts: &[Vert], tris: &[[usize; 3]]) -> Mesh {
    let mut pos_out: Vec<[f32; 3]>  = Vec::new();
    let mut nor_out: Vec<[f32; 3]>  = Vec::new();
    let mut uv_out:  Vec<[f32; 2]>  = Vec::new();
    let mut idx_out: Vec<u32>        = Vec::new();
    let mut cache:   HashMap<[i32; 8], u32> = HashMap::new();

    for tri in tris {
        for &vi in tri {
            if vi >= verts.len() { continue; }
            let v = &verts[vi];
            let key = quantize_vert(v);
            let new_idx = *cache.entry(key).or_insert_with(|| {
                let i = pos_out.len() as u32;
                pos_out.push(v.pos);
                nor_out.push(v.nor);
                uv_out.push(v.uv);
                i
            });
            idx_out.push(new_idx);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos_out);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,   nor_out);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0,     uv_out);
    mesh.insert_indices(Indices::U32(idx_out));
    mesh
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn extract_verts(mesh: &Mesh) -> Vec<Vert> {
    let pos = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3())
        .map(|s| s.to_vec())
        .unwrap_or_default();
    let nor = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|a| a.as_float3())
        .map(|s| s.to_vec())
        .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; pos.len()]);
    let uv = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(v)) => v.clone(),
        _ => vec![[0.0, 0.0]; pos.len()],
    };
    (0..pos.len())
        .map(|i| Vert { pos: pos[i], nor: nor[i], uv: uv[i] })
        .collect()
}

fn get_triangles(mesh: &Mesh) -> Vec<[usize; 3]> {
    match mesh.indices() {
        Some(Indices::U16(v)) => v.chunks(3)
            .filter(|c| c.len() == 3)
            .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
            .collect(),
        Some(Indices::U32(v)) => v.chunks(3)
            .filter(|c| c.len() == 3)
            .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
            .collect(),
        None => Vec::new(),
    }
}
