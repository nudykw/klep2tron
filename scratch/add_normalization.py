import sys
import os

file_path = "crates/client_core/src/actor_editor/systems_logic.rs"
content = open(file_path).read()

# 1. Update actor_import_processing_system to add AwaitingNormalization
# And fix OBJ spawning to also use AwaitingNormalization for consistency

old_obj_spawn = """                commands.spawn((
                    PbrBundle {
                        mesh: mesh_handle.clone(),
                        material: materials.add(StandardMaterial {
                            base_color: Color::WHITE,
                            ..default()
                        }),
                        transform: Transform::from_scale(Vec3::splat(scale))
                            .with_translation(Vec3::from(-aabb.center) * scale + Vec3::Y * (aabb.half_extents.y * scale)),
                        ..default()
                    },
                    super::ActorEditorEntity,
                    OriginalMeshComponent(mesh_handle),
                ));"""

new_obj_spawn = """                commands.spawn((
                    PbrBundle {
                        mesh: mesh_handle.clone(),
                        material: materials.add(StandardMaterial {
                            base_color: Color::WHITE,
                            ..default()
                        }),
                        ..default()
                    },
                    super::ActorEditorEntity,
                    super::AwaitingNormalization,
                    OriginalMeshComponent(mesh_handle),
                ));"""

old_scene_spawn = """             commands.spawn((
                SceneBundle {
                    scene: pending.handle.clone().unwrap(),
                    ..default()
                },
                super::ActorEditorEntity,
            ));"""

new_scene_spawn = """             commands.spawn((
                SceneBundle {
                    scene: pending.handle.clone().unwrap(),
                    ..default()
                },
                super::ActorEditorEntity,
                super::AwaitingNormalization,
            ));"""

content = content.replace(old_obj_spawn, new_obj_spawn)
content = content.replace(old_scene_spawn, new_scene_spawn)

# 2. Add the normalization system at the end
normalization_system = """
pub fn normalization_system(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform), With<super::AwaitingNormalization>>,
    children_query: Query<&Children>,
    mesh_query: Query<(&bevy::render::primitives::Aabb, &GlobalTransform), With<Handle<Mesh>>>,
) {
    for (root_entity, root_transform) in query.iter() {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        let mut found = false;

        // Recursively find all AABBs in world space
        let mut stack = vec![root_entity];
        while let Some(entity) = stack.pop() {
            if let Ok((aabb, transform)) = mesh_query.get(entity) {
                // Convert AABB to world space
                let world_aabb = bevy::render::primitives::Aabb {
                    center: transform.compute_matrix().transform_point3a(aabb.center),
                    half_extents: transform.compute_matrix().transform_vector3a(aabb.half_extents).abs(),
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
                let root_world_pos = root_transform.translation();
                
                // We want to move the model so its center is at origin, 
                // and then shift it up so it stands on the ground (Y=0)
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
}
"""

with open(file_path, "w") as f:
    f.write(content + normalization_system)
"""

# Wait, I realized I should not use python for this complex edit because I might have missed something.
# I will use write_to_file for the whole file again to be 100% sure.
