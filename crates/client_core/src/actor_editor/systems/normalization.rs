use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::mesh::VertexAttributeValues;
use super::super::{NormalizationState, ImportProgress, EditorStatus, ActorBounds, AwaitingNormalization, OriginalMeshComponent, EditorHelper};

pub fn normalization_system(
    mut commands: Commands,
    query: Query<Entity, With<AwaitingNormalization>>,
    mut state_query: Query<(Entity, &mut NormalizationState)>,
    children_query: Query<&Children>,
    parent_query: Query<&Parent>,
    transform_query: Query<&Transform>,
    mesh_query: Query<(Entity, &Aabb, &GlobalTransform, &Handle<Mesh>, Option<&Name>)>,
    meshes: Res<Assets<Mesh>>,
    mut progress: ResMut<ImportProgress>,
    mut status: ResMut<EditorStatus>,
    mut scaling_settings: ResMut<crate::actor_editor::ScalingSettings>,
) {
    for root_entity in query.iter() {
        let mut stack = vec![root_entity];
        let mut entities_to_process = Vec::new();
        while let Some(entity) = stack.pop() {
            entities_to_process.push(entity);
            if let Ok(children) = children_query.get(entity) { for child in children.iter() { stack.push(*child); } }
        }
        commands.entity(root_entity).remove::<AwaitingNormalization>();
        commands.entity(root_entity).insert(NormalizationState { entities_to_process,                processed_count: 0, 
                min: Vec3::splat(f32::MAX), 
                max: Vec3::splat(f32::MIN), 
                found_meshes: Vec::new(),
                total_original_polys: 0,
            });
        progress.0 = 0.7; *status = EditorStatus::Processing;
    }

    for (root_entity, mut state) in state_query.iter_mut() {
        let chunk_size = 50;
        let mut processed_this_frame = 0;
        while processed_this_frame < chunk_size && state.processed_count < state.entities_to_process.len() {
            let entity = state.entities_to_process[state.processed_count];
            if let Ok((_entity, aabb, _transform, _mesh_handle, name_opt)) = mesh_query.get(entity) {
                let name = name_opt.map(|n| n.as_str().to_lowercase()).unwrap_or_default();
                let is_env = name.contains("shadow") || name.contains("floor") || name.contains("plane") ||
                             name.contains("grid") || name.contains("background");
                
                if !is_env {
                    let mut current_matrix = Mat4::IDENTITY;
                    let mut curr = entity;
                    let mut found_root = false;
                    
                    for _ in 0..32 { // Safety limit
                        if curr == root_entity { found_root = true; break; }
                        if let Ok(transform) = transform_query.get(curr) {
                            current_matrix = transform.compute_matrix() * current_matrix;
                        }
                        if let Ok(parent) = parent_query.get(curr) {
                            curr = parent.get();
                        } else {
                            break;
                        }
                    }

                    if found_root {
                        let min = aabb.min();
                        let max = aabb.max();
                        let corners = [
                            Vec3::new(min.x, min.y, min.z),
                            Vec3::new(min.x, min.y, max.z),
                            Vec3::new(min.x, max.y, min.z),
                            Vec3::new(min.x, max.y, max.z),
                            Vec3::new(max.x, min.y, min.z),
                            Vec3::new(max.x, min.y, max.z),
                            Vec3::new(max.x, max.y, min.z),
                            Vec3::new(max.x, max.y, max.z),
                        ];

                        let mut mesh_min = Vec3::splat(f32::MAX);
                        let mut mesh_max = Vec3::splat(f32::MIN);
                        for corner in corners {
                            let pos = current_matrix.transform_point3(corner);
                            mesh_min = mesh_min.min(pos);
                            mesh_max = mesh_max.max(pos);
                        }

                        state.min = state.min.min(mesh_min);
                        state.max = state.max.max(mesh_max);
                        state.found_meshes.push((entity, _mesh_handle.clone()));
                        
                        if let Some(mesh) = meshes.get(_mesh_handle) {
                            if let Some(indices) = mesh.indices() {
                                state.total_original_polys += indices.len() / 3;
                            } else if let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                                state.total_original_polys += p.len() / 3;
                            }
                        }
                    }
                }
            }
            state.processed_count += 1; processed_this_frame += 1;
        }
        let ratio = state.processed_count as f32 / state.entities_to_process.len() as f32;
        progress.0 = 0.7 + ratio * 0.25;
        if state.processed_count >= state.entities_to_process.len() {
            if !state.found_meshes.is_empty() {
                let center = (state.min + state.max) / 2.0;
                let size = state.max - state.min;
                let max_dim = size.x.max(size.y).max(size.z);
                if max_dim > 0.0 {
                    let scale = 2.0 / max_dim;
                    let offset = -center;
                    let y_offset = size.y * 0.5;
                    let translation = (offset + Vec3::Y * y_offset) * scale;
                    
                    let pivot = commands.spawn((
                        SpatialBundle {
                            transform: Transform {
                                translation,
                                rotation: Quat::IDENTITY,
                                scale: Vec3::splat(scale),
                            },
                            ..default()
                        },
                        EditorHelper,
                        Name::new("NormalizationPivot"),
                    )).id();

                    if let Ok(children) = children_query.get(root_entity) {
                        for child in children.iter() {
                            commands.entity(pivot).add_child(*child);
                        }
                    }
                    commands.entity(root_entity).add_child(pivot);

                    let s = size * scale;
                    commands.entity(root_entity).insert(ActorBounds { 
                        min: Vec3::new(-s.x * 0.5, 0.0, -s.z * 0.5), 
                        max: Vec3::new(s.x * 0.5, s.y, s.z * 0.5),
                        original_polys: state.total_original_polys,
                    });

                    // Sync Scaling Settings - multiply normalized size by root scale to get world size
                    let root_scale = transform_query.get(root_entity).map(|t| t.scale).unwrap_or(Vec3::ONE);
                    scaling_settings.width = s.x * root_scale.x;
                    scaling_settings.height = s.y * root_scale.y;
                    scaling_settings.length = s.z * root_scale.z;

                    for (entity, handle) in &state.found_meshes { 
                        commands.entity(*entity).insert(OriginalMeshComponent(handle.clone())); 
                    }
                    info!("Normalization complete for {} meshes", state.found_meshes.len());
                }
            }
            commands.entity(root_entity).remove::<NormalizationState>();
            if !state.found_meshes.is_empty() {
                progress.0 = 0.95; // Keep visible until first slice
            } else {
                progress.0 = 1.0; *status = EditorStatus::Ready;
            }
        }
    }
}
