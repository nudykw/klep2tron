use bevy::prelude::*;

pub struct RayHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub triangle_index: usize,
}

pub fn ray_mesh_intersection(
    ray_origin: Vec3,
    ray_dir: Vec3,
    mesh: &Mesh,
    transform: &GlobalTransform,
) -> Option<RayHit> {
    let matrix = transform.compute_matrix();
    let inv_matrix = matrix.inverse();
    
    // Transform ray to local space
    let local_origin = inv_matrix.transform_point3(ray_origin);
    let local_dir = inv_matrix.transform_vector3(ray_dir).normalize();
    
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?.as_float3()?;
    let indices = mesh.indices()?;
    
    let mut best_t: Option<f32> = None;
    let mut best_normal: Option<Vec3> = None;
    let mut best_index: Option<usize> = None;

    let triangles = match indices {
        bevy::render::mesh::Indices::U16(vec) => vec.chunks(3).map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect::<Vec<_>>(),
        bevy::render::mesh::Indices::U32(vec) => vec.chunks(3).map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect::<Vec<_>>(),
    };

    for (tri_idx, triangle) in triangles.iter().enumerate() {
        let v0 = Vec3::from(positions[triangle[0]]);
        let v1 = Vec3::from(positions[triangle[1]]);
        let v2 = Vec3::from(positions[triangle[2]]);
        
        if let Some(t) = ray_triangle_intersection(local_origin, local_dir, v0, v1, v2) {
            if best_t.is_none() || t < best_t.unwrap() {
                best_t = Some(t);
                best_normal = Some((v1 - v0).cross(v2 - v0).normalize());
                best_index = Some(tri_idx);
            }
        }
    }
    
    if let (Some(t), Some(normal), Some(index)) = (best_t, best_normal, best_index) {
        let world_point = matrix.transform_point3(local_origin + local_dir * t);
        let world_normal = transform.to_scale_rotation_translation().1 * normal;
        let world_distance = ray_origin.distance(world_point);
        
        Some(RayHit {
            point: world_point,
            normal: world_normal,
            distance: world_distance,
            triangle_index: index,
        })
    } else {
        None
    }
}

fn ray_triangle_intersection(
    orig: Vec3,
    dir: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<f32> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let pvec = dir.cross(edge2);
    let det = edge1.dot(pvec);
    
    if det.abs() < 1e-6 { return None; }
    let inv_det = 1.0 / det;
    
    let tvec = orig - v0;
    let u = tvec.dot(pvec) * inv_det;
    if u < 0.0 || u > 1.0 { return None; }
    
    let qvec = tvec.cross(edge1);
    let v = dir.dot(qvec) * inv_det;
    if v < 0.0 || u + v > 1.0 { return None; }
    
    let t = edge2.dot(qvec) * inv_det;
    if t > 1e-6 { Some(t) } else { None }
}
