use bevy::prelude::*;
use super::super::{SocketSettings, ActorPart, geometry::raycast, MainEditorCamera, ActorSocket, SocketDefinition, HoveredSocketData};
use super::super::ui::inspector::SocketAddModeButton;
use super::super::ui::inspector::types::{SelectedSocket, MultiSelectionState};

pub fn socket_picking_system(
    mut settings: ResMut<SocketSettings>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainEditorCamera>>,
    part_query: Query<(Entity, &ActorPart, &Handle<Mesh>, &GlobalTransform)>,
    meshes: Res<Assets<Mesh>>,
) {
    if !settings.is_adding {
        settings.hovered_data = None;
        return;
    }

    let Ok(window) = window_query.get_single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };

    let Some(cursor_pos) = window.cursor_position() else {
        settings.hovered_data = None;
        return;
    };

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        settings.hovered_data = None;
        return;
    };

    let mut best_hit: Option<HoveredSocketData> = None;
    let mut min_dist = f32::MAX;

    for (entity, part_type, mesh_handle, transform) in part_query.iter() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            // Use custom ray-mesh intersection
            if let Some(hit) = raycast::ray_mesh_intersection(ray.origin, ray.direction.into(), mesh, transform) {
                if hit.distance < min_dist {
                    min_dist = hit.distance;
                    best_hit = Some(HoveredSocketData {
                        part_entity: entity,
                        part_type: *part_type,
                        point: hit.point,
                        normal: hit.normal,
                    });
                }
            }
        }
    }

    settings.hovered_data = best_hit;
}

pub fn socket_spawn_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<SocketSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: ResMut<super::super::ui::inspector::SelectedSocket>,
    mut action_stack: ResMut<super::super::systems::undo_redo::ActionStack>,
    actor_root_query: Query<(Entity, &GlobalTransform), With<crate::actor_editor::Actor3DRoot>>,
    _part_query: Query<&GlobalTransform, With<ActorPart>>,
    socket_query: Query<&ActorSocket>,
) {
    if !settings.is_adding { return; }
    
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(data) = settings.hovered_data.clone() {
            let Ok((actor_root, root_transform)) = actor_root_query.get_single() else { 
                info!("Socket Spawn Failed: Actor3DRoot not found");
                return; 
            };
            
            // Calculate world rotation aligned with normal
            let world_rotation = Quat::from_rotation_arc(Vec3::Y, data.normal);
            let offset_point = data.point + data.normal * 0.01;
            
            // Calculate local transform relative to the actor root
            let inv_matrix = root_transform.compute_matrix().inverse();
            let local_point = inv_matrix.transform_point3(offset_point);
            let local_rotation = root_transform.to_scale_rotation_translation().1.inverse() * world_rotation;

            let name = find_next_socket_name(data.part_type, &socket_query);

            let socket_entity = commands.spawn((
                PbrBundle {
                    // Base: Torus
                    mesh: meshes.add(Mesh::from(bevy::math::primitives::Torus::new(0.01, 0.04))),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(0.2, 0.8, 0.2),
                        metallic: 0.8,
                        perceptual_roughness: 0.2,
                        depth_bias: 500.0,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    }),
                    transform: Transform::from_translation(local_point).with_rotation(local_rotation),
                    ..default()
                },
                ActorSocket {
                    definition: SocketDefinition {
                        name: name.clone(),
                        part: data.part_type,
                        position: local_point,
                        rotation: local_rotation,
                        comment: String::new(),
                        color: Color::srgb(0.2, 0.8, 0.2),
                        effect: None,
                        effect_preset: None,
                    }
                },
                bevy_mod_picking::PickableBundle::default(),
                Name::new("ActorSocket"),
                crate::actor_editor::ActorEditorEntity, // Mark as editor entity so it's not cleaned up accidentally
            )).with_children(|parent| {
                // ... (children same as before)
                parent.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::math::primitives::Cylinder::new(0.005, 0.15))),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 0.0),
                        unlit: true,
                        depth_bias: 500.0,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.075, 0.0),
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::math::primitives::Cone { radius: 0.015, height: 0.05 })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 0.0),
                        unlit: true,
                        depth_bias: 500.0,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.15, 0.0),
                    ..default()
                });
            }).id();
            
            commands.entity(actor_root).add_child(socket_entity);
            selected.0 = vec![socket_entity];
            settings.is_adding = false; // Turn off "plus" mode after spawn
            
            // Record for Undo/Redo
            action_stack.push(Box::new(super::super::systems::undo_redo::AddSocketCommand {
                entity: socket_entity,
                definition: SocketDefinition {
                    name,
                    part: data.part_type,
                    position: local_point,
                    rotation: local_rotation,
                    comment: String::new(),
                    color: Color::srgb(0.2, 0.8, 0.2),
                    effect: None,
                    effect_preset: None,
                },
            }));

            info!("Spawned and Selected new socket: {:?} ({:?})", socket_entity, data.part_type);
        }
    }
}

fn find_next_socket_name(part: ActorPart, query: &Query<&ActorSocket>) -> String {
    let mut indices = Vec::new();
    let prefix = format!("Socket_{:?}_", part);
    
    for socket in query.iter() {
        if socket.definition.part == part && socket.definition.name.starts_with(&prefix) {
            if let Ok(idx) = socket.definition.name[prefix.len()..].parse::<u32>() {
                indices.push(idx);
            }
        }
    }
    
    let mut next_idx = 1;
    indices.sort();
    for idx in indices {
        if idx == next_idx {
            next_idx += 1;
        } else if idx > next_idx {
            break;
        }
    }
    
    format!("{}{}", prefix, next_idx)
}

pub fn socket_ui_interaction_system(
    mut settings: ResMut<SocketSettings>,
    query: Query<&Interaction, (Changed<Interaction>, With<SocketAddModeButton>)>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            settings.is_adding = !settings.is_adding;
        }
    }
}

pub fn socket_button_visuals_system(
    settings: Res<SocketSettings>,
    mut query: Query<(&mut BackgroundColor, &Interaction), With<SocketAddModeButton>>,
) {
    for (mut bg, interaction) in query.iter_mut() {
        if settings.is_adding {
            *bg = Color::srgba(0.0, 1.0, 0.5, 0.4).into();
        } else if *interaction == Interaction::Hovered {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.1).into();
        } else {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
        }
    }
}
pub fn socket_3d_selection_system(
    mut selected: ResMut<SelectedSocket>,
    mut multi_state: ResMut<MultiSelectionState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut events: EventReader<bevy_mod_picking::prelude::Pointer<bevy_mod_picking::prelude::Click>>,
    socket_query: Query<Entity, With<super::super::ActorSocket>>,
) {
    for event in events.read() {
        if socket_query.get(event.target).is_ok() {
            let entity = event.target;
            
            if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
                if let Some(pos) = selected.0.iter().position(|&e| e == entity) {
                    selected.0.remove(pos);
                } else {
                    selected.0.push(entity);
                }
                multi_state.last_selected = Some(entity);
            } else {
                selected.0 = vec![entity];
                multi_state.last_selected = Some(entity);
            }
        }
    }
}



pub fn draw_socket_previews_system(
    settings: Res<SocketSettings>,
    mut gizmos: Gizmos,
) {
    if let Some(data) = &settings.hovered_data {
        let p = data.point;
        let n = data.normal;
        
        // Draw a ghost of the socket
        let color = Color::srgba(1.0, 1.0, 1.0, 0.4);
        
        // "Torus" approximation using 2 circles
        if let Ok(dir) = Dir3::new(n) {
            gizmos.circle(p + n * 0.01, dir, 0.04, color);
            gizmos.circle(p + n * 0.01, dir, 0.035, color);
        }
        
        // "Pin" direction
        gizmos.line(p, p + n * 0.15, Color::srgba(1.0, 1.0, 0.0, 0.5));
        
        // Small dot at center
        gizmos.sphere(p, Quat::IDENTITY, 0.005, Color::WHITE);
    }
}

pub fn socket_deletion_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedSocket>,
    mut action_stack: ResMut<super::super::systems::undo_redo::ActionStack>,
    socket_query: Query<&ActorSocket>,
) {
    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        let to_delete = selected.0.clone();
        for entity in to_delete {
            if let Ok(socket) = socket_query.get(entity) {
                // Record
                action_stack.push(Box::new(super::super::systems::undo_redo::DeleteSocketCommand {
                    entity,
                    definition: socket.definition.clone(),
                }));
                
                // Hide and remove component instead of despawn to keep Entity ID stable for undo
                commands.entity(entity).remove::<ActorSocket>().insert(Visibility::Hidden);
            }
        }
        selected.0.clear();
    }
}

pub fn socket_restoration_system(
    mut commands: Commands,
    mut pending: ResMut<super::super::PendingSockets>,
    status: Res<super::super::EditorStatus>,
    actor_root_query: Query<Entity, With<crate::actor_editor::Actor3DRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *status != super::super::EditorStatus::Ready || pending.0.is_empty() { return; }
    
    let Ok(actor_root) = actor_root_query.get_single() else { return; };
    
    info!("Restoring {} sockets...", pending.0.len());
    
    for def in pending.0.drain(..) {
        let socket_entity = commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(bevy::math::primitives::Torus::new(0.01, 0.04))),
                material: materials.add(StandardMaterial {
                    base_color: def.color,
                    metallic: 0.8,
                    perceptual_roughness: 0.2,
                    depth_bias: 500.0,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                transform: Transform::from_translation(def.position).with_rotation(def.rotation),
                ..default()
            },
            super::super::ActorSocket { definition: def.clone() },
            bevy_mod_picking::PickableBundle::default(),
            Name::new("ActorSocket"),
            crate::actor_editor::ActorEditorEntity,
        )).with_children(|parent| {
            // Forward indicator
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(bevy::math::primitives::Cylinder::new(0.005, 0.15))),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 0.0),
                    unlit: true,
                    depth_bias: 500.0,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 0.075, 0.0),
                ..default()
            });
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(bevy::math::primitives::Cone { radius: 0.015, height: 0.05 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 0.0),
                    unlit: true,
                    depth_bias: 500.0,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 0.15, 0.0),
                ..default()
            });
        }).id();
        
        commands.entity(actor_root).add_child(socket_entity);
    }
}

