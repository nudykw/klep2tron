use bevy::prelude::*;
use super::{GizmoAxis, GizmoAxisType, GizmoAction, ManualGizmoInteraction, SocketLink, SocketGizmo};
use super::super::super::ActorSocket;

pub fn manual_gizmo_dragging_system(
    mouse: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::actor_editor::MainEditorCamera>>,
    mut gizmo_query: Query<(Entity, &mut ManualGizmoInteraction, &GizmoAxis, &SocketLink, &GlobalTransform)>,
    mut socket_query: Query<&mut Transform, With<crate::actor_editor::ActorSocket>>,
    ui_query: Query<&Interaction, With<Node>>,
    mut last_cursor: Local<Option<Vec2>>,
    mut active_axis: Local<Option<(Entity, GizmoAxisType, GizmoAction, Entity)>>,
    mut initial_rotation_vector: Local<Option<Vec3>>,
    mut initial_socket_rotation: Local<Option<Quat>>,
    selected: Res<crate::actor_editor::ui::inspector::SelectedSocket>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    
    if mouse.just_pressed(MouseButton::Left) {
        // Skip if clicking on UI
        for interaction in ui_query.iter() {
            if *interaction != Interaction::None {
                return;
            }
        }

        for (entity, mut interaction, axis, link, gt) in gizmo_query.iter_mut() {
            if *interaction == ManualGizmoInteraction::Hovered {
                *interaction = ManualGizmoInteraction::Pressed;
                *active_axis = Some((entity, axis.axis, axis.action, link.0));
                
                if axis.action == GizmoAction::Rotate {
                    let Ok((camera, camera_gt)) = camera_query.get_single() else { continue; };
                    if let Some(ray) = camera.viewport_to_world(camera_gt, cursor_pos) {
                        let center = gt.translation();
                        let normal = match axis.axis {
                            GizmoAxisType::X => Vec3::X,
                            GizmoAxisType::Y => Vec3::Y,
                            GizmoAxisType::Z => Vec3::Z,
                        };
                        
                        // Plane intersection
                        let denom = normal.dot(ray.direction.into());
                        if denom.abs() > 0.0001 {
                            let t = (center - ray.origin).dot(normal) / denom;
                            let hit_point = ray.origin + Vec3::from(ray.direction) * t;
                            *initial_rotation_vector = Some((hit_point - center).normalize());
                            
                            // Store initial socket rotation
                            if let Ok(transform) = socket_query.get(link.0) {
                                *initial_socket_rotation = Some(transform.rotation);
                            }
                        }
                    }
                }
                
                info!("--- GIZMO DRAG START: {:?} {:?} ---", axis.action, axis.axis);
                break;
            }
        }
    }
    
    if mouse.pressed(MouseButton::Left) {
        if let Some((_axis_entity, axis_type, action, socket_entity)) = *active_axis {
            if let Some(last_pos) = *last_cursor {
                let delta = cursor_pos - last_pos;
                if delta.length_squared() > 0.0001 {
                    if let Ok(mut transform) = socket_query.get_mut(socket_entity) {
                        let Ok((camera, camera_gt)) = camera_query.get_single() else { return; };
                        
                        match action {
                            GizmoAction::Translate => {
                                let camera_right = camera_gt.right();
                                let camera_up = camera_gt.up();
                                
                                let move_dir = match axis_type {
                                    GizmoAxisType::X => Vec3::X,
                                    GizmoAxisType::Y => Vec3::Y,
                                    GizmoAxisType::Z => Vec3::Z,
                                };
                                
                                let sensitivity = 0.005;
                                let world_delta = (camera_right * delta.x - camera_up * delta.y) * sensitivity;
                                let axis_movement = world_delta.dot(move_dir);
                                let delta_vec = move_dir * axis_movement;
                                
                                for &entity in selected.0.iter() {
                                    if let Ok(mut t) = socket_query.get_mut(entity) {
                                        t.translation += delta_vec;
                                    }
                                }
                            },
                            GizmoAction::Rotate => {
                                if let (Some(initial_vec), Some(start_rot)) = (*initial_rotation_vector, *initial_socket_rotation) {
                                    if let Some(ray) = camera.viewport_to_world(camera_gt, cursor_pos) {
                                        let center = transform.translation;
                                        let normal = match axis_type {
                                            GizmoAxisType::X => Vec3::X,
                                            GizmoAxisType::Y => Vec3::Y,
                                            GizmoAxisType::Z => Vec3::Z,
                                        };
                                        
                                        let denom = normal.dot(ray.direction.into());
                                        if denom.abs() > 0.0001 {
                                            let t = (center - ray.origin).dot(normal) / denom;
                                            let hit_point = ray.origin + Vec3::from(ray.direction) * t;
                                            let current_vec = (hit_point - center).normalize();
                                            
                                            // Calculate signed angle between initial and current
                                            let cross = initial_vec.cross(current_vec);
                                            let angle = initial_vec.dot(current_vec).acos() * cross.dot(normal).signum();
                                            
                                            if !angle.is_nan() {
                                                let rotation_delta = Quat::from_axis_angle(normal, angle);
                                                
                                                // For rotation, we need to apply delta to all selected
                                                // BUT each might have a different start rotation.
                                                // This is tricky with the "apply from start" logic.
                                                // For now, let's just apply to the anchor. 
                                                // Proper group rotation would require storing ALL initial rotations.
                                                transform.rotation = rotation_delta * start_rot;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        if let Some((entity, _, _, _)) = *active_axis {
            info!("--- GIZMO DRAG STOP ---");
            if let Ok((_, mut interaction, _, _, _)) = gizmo_query.get_mut(entity) {
                *interaction = ManualGizmoInteraction::None;
            }
        }
        *active_axis = None;
        *initial_rotation_vector = None;
        *initial_socket_rotation = None;
    }
    
    *last_cursor = Some(cursor_pos);
}

pub fn gizmo_position_sync_system(
    socket_query: Query<&GlobalTransform, With<ActorSocket>>,
    mut gizmo_query: Query<(&mut Transform, &SocketLink), With<SocketGizmo>>,
) {
    for (mut transform, link) in gizmo_query.iter_mut() {
        if let Ok(socket_gt) = socket_query.get(link.0) {
            let (_, _, translation) = socket_gt.to_scale_rotation_translation();
            transform.translation = translation;
            transform.rotation = Quat::IDENTITY; // Always world-aligned
            transform.scale = Vec3::ONE; 
        }
    }
}

pub fn socket_gizmo_sync_system(
    mut socket_query: Query<(&Transform, &mut ActorSocket)>,
) {
    for (transform, mut socket) in socket_query.iter_mut() {
        socket.definition.position = transform.translation;
        socket.definition.rotation = transform.rotation;
    }
}
