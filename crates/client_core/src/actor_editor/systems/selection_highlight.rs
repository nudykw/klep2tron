use bevy::prelude::*;
use super::super::SelectedTriangles;

pub fn selection_highlight_system(
    part_query: Query<(&Handle<Mesh>, &GlobalTransform, &SelectedTriangles)>,
    meshes: Res<Assets<Mesh>>,
    mut gizmos: Gizmos,
) {
    for (mesh_handle, transform, selected) in part_query.iter() {
        if selected.indices.is_empty() { continue; }
        
        if let Some(mesh) = meshes.get(mesh_handle) {
            let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap();
            let indices = mesh.indices().unwrap();
            
            // Collect all triangles once
            // Optimization: we could cache this or use the mesh directly if we had a way to access triangles efficiently
            let tri_data: Vec<[usize; 3]> = match indices {
                bevy::render::mesh::Indices::U16(vec) => vec.chunks(3).map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect(),
                bevy::render::mesh::Indices::U32(vec) => vec.chunks(3).map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect(),
            };

            let color = Color::srgb(0.0, 1.0, 1.0); // Cyan glow for selected triangles

            for &idx in &selected.indices {
                if let Some(tri) = tri_data.get(idx) {
                    let v0 = transform.transform_point(Vec3::from(positions[tri[0]]));
                    let v1 = transform.transform_point(Vec3::from(positions[tri[1]]));
                    let v2 = transform.transform_point(Vec3::from(positions[tri[2]]));
                    
                    // Draw edges with slight offset to avoid Z-fighting with mesh
                    // Or just use a negative depth bias in gizmo config (handled globally usually)
                    gizmos.line(v0, v1, color);
                    gizmos.line(v1, v2, color);
                    gizmos.line(v2, v0, color);
                    
                    // Optional: Draw a small "X" or cross in the middle for more visibility
                    // let center = (v0 + v1 + v2) / 3.0;
                    // gizmos.sphere(center, Quat::IDENTITY, 0.002, color);
                }
            }
        }
    }
}
