import sys
import os

file_path = "crates/client_core/src/actor_editor/systems_logic.rs"
content = open(file_path).read()

# 1. Update polycount_update_system to be recursive
old_polycount = """pub fn polycount_update_system(
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Handle<Mesh>, With<super::ActorEditorEntity>>,
    mut text_query: Query<&mut Text, With<super::widgets::PolycountText>>,
) {
    let mut total_polys = 0;
    for handle in mesh_query.iter() {
        if let Some(mesh) = meshes.get(handle) {
            if let Some(indices) = mesh.indices() {
                total_polys += indices.len() / 3;
            } else {
                // If no indices, assume it's a triangle list and use vertex count
                if let Some(pos) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    total_polys += pos.len() / 3;
                }
            }
        }
    }
    
    if let Ok(mut text) = text_query.get_single_mut() {
        let new_val = format!("POLYS: {}", total_polys);
        if text.sections[0].value != new_val {
            text.sections[0].value = new_val;
        }
    }
}"""

new_polycount = """pub fn polycount_update_system(
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Handle<Mesh>>,
    root_query: Query<Entity, With<super::ActorEditorEntity>>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut Text, With<super::widgets::PolycountText>>,
) {
    let mut total_polys = 0;
    
    for root_entity in root_query.iter() {
        let mut stack = vec![root_entity];
        while let Some(entity) = stack.pop() {
            if let Ok(handle) = mesh_query.get(entity) {
                if let Some(mesh) = meshes.get(handle) {
                    if let Some(indices) = mesh.indices() {
                        total_polys += indices.len() / 3;
                    } else if let Some(pos) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                        total_polys += pos.len() / 3;
                    }
                }
            }
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    stack.push(*child);
                }
            }
        }
    }
    
    if let Ok(mut text) = text_query.get_single_mut() {
        let new_val = format!("POLYS: {}", total_polys);
        if text.sections[0].value != new_val {
            text.sections[0].value = new_val;
        }
    }
}"""

# 2. Update normalization_system to attach OriginalMeshComponent
old_normalization = """pub fn normalization_system(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform), With<super::AwaitingNormalization>>,
    children_query: Query<&Children>,
    mesh_query: Query<(&Aabb, &GlobalTransform), With<Handle<Mesh>>>,
) {
    for (root_entity, _root_transform) in query.iter() {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        let mut found = false;

        // Recursively find all AABBs in world space
        let mut stack = vec![root_entity];
        while let Some(entity) = stack.pop() {
            if let Ok((aabb, transform)) = mesh_query.get(entity) {
                let matrix = transform.compute_matrix();
                let world_aabb = Aabb {
                    center: matrix.transform_point3a(aabb.center),
                    half_extents: matrix.transform_vector3a(aabb.half_extents).abs(),
                };
                
                let aabb_min = Vec3::from(world_aabb.center - world_aabb.half_extents);
                let aabb_max = Vec3::from(world_aabb.center + world_aabb.half_extents);
                
                min = min.min(aabb_min);
                max = max.max(aabb_max);
                found = true;
            }
            
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    stack.push(*child);
                }
            }
        }

        if found {
            let center = (min + max) / 2.0;
            let size = max - min;
            let max_dim = size.x.max(size.y).max(size.z);
            
            if max_dim > 0.0 {
                let scale = 2.0 / max_dim;
                
                // We want to move the model so its center is at origin, 
                // and then shift it up so it stands on the ground (Y=0)
                // Since this is the root, we apply the inverse of the world center to it
                let offset = -center;
                let y_offset = size.y * 0.5; // Lift up to stand on Y=0
                
                commands.entity(root_entity).insert(Transform {
                    translation: (offset + Vec3::Y * y_offset) * scale,
                    scale: Vec3::splat(scale),
                    rotation: Quat::IDENTITY,
                });
                
                commands.entity(root_entity).remove::<super::AwaitingNormalization>();
            }
        }
    }
}"""

new_normalization = """pub fn normalization_system(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform), With<super::AwaitingNormalization>>,
    children_query: Query<&Children>,
    mesh_query: Query<(&Aabb, &GlobalTransform, &Handle<Mesh>)>,
) {
    for (root_entity, _root_transform) in query.iter() {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        let mut found = false;
        let mut found_meshes = Vec::new();

        // Recursively find all AABBs in world space
        let mut stack = vec![root_entity];
        while let Some(entity) = stack.pop() {
            if let Ok((aabb, transform, mesh_handle)) = mesh_query.get(entity) {
                let matrix = transform.compute_matrix();
                let world_aabb = Aabb {
                    center: matrix.transform_point3a(aabb.center),
                    half_extents: matrix.transform_vector3a(aabb.half_extents).abs(),
                };
                
                let aabb_min = Vec3::from(world_aabb.center - world_aabb.half_extents);
                let aabb_max = Vec3::from(world_aabb.center + world_aabb.half_extents);
                
                min = min.min(aabb_min);
                max = max.max(aabb_max);
                found = true;
                found_meshes.push((entity, mesh_handle.clone()));
            }
            
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    stack.push(*child);
                }
            }
        }

        if found {
            let center = (min + max) / 2.0;
            let size = max - min;
            let max_dim = size.x.max(size.y).max(size.z);
            
            if max_dim > 0.0 {
                let scale = 2.0 / max_dim;
                let offset = -center;
                let y_offset = size.y * 0.5;
                
                commands.entity(root_entity).insert(Transform {
                    translation: (offset + Vec3::Y * y_offset) * scale,
                    scale: Vec3::splat(scale),
                    rotation: Quat::IDENTITY,
                });
                
                // Attach OriginalMeshComponent to each entity with a mesh
                for (entity, handle) in found_meshes {
                    commands.entity(entity).insert(super::OriginalMeshComponent(handle));
                }
                
                commands.entity(root_entity).remove::<super::AwaitingNormalization>();
            }
        }
    }
}"""

content = content.replace(old_polycount, new_polycount)
content = content.replace(old_normalization, new_normalization)

with open(file_path, "w") as f:
    f.write(content)
"""
# I will use write_to_file for the whole file again to be 100% sure.
