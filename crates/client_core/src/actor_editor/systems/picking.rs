use bevy::prelude::*;
use super::super::{SocketSettings, ActorPart, geometry::raycast, MainEditorCamera, ActorSocket, SocketDefinition, HoveredSocketData};
use super::super::ui_inspector::SocketAddModeButton;

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
    mut selected: ResMut<super::super::ui_inspector::SelectedSocket>,
    part_query: Query<&GlobalTransform, With<ActorPart>>,
) {
    if !settings.is_adding { return; }
    
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(data) = settings.hovered_data.clone() {
            let Ok(part_global_transform) = part_query.get(data.part_entity) else { return; };
            
            // Calculate world rotation aligned with normal
            let world_rotation = Quat::from_rotation_arc(Vec3::Y, data.normal);
            let offset_point = data.point + data.normal * 0.01;
            
            // Calculate local transform relative to the parent part
            let inv_matrix = part_global_transform.compute_matrix().inverse();
            let local_point = inv_matrix.transform_point3(offset_point);
            let local_rotation = part_global_transform.to_scale_rotation_translation().1.inverse() * world_rotation;

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
                        name: format!("Socket_{:?}_{}", data.part_type, rand_id()),
                        part: data.part_type,
                        position: local_point,
                        rotation: local_rotation,
                    }
                },
                bevy_mod_picking::PickableBundle::default(),
                Name::new("ActorSocket"),
            )).with_children(|parent| {
                // Pin: Points along the normal (Y axis of torus mesh)
                parent.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::math::primitives::Cylinder::new(0.005, 0.1))),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 0.0),
                        unlit: true,
                        depth_bias: 500.0,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.05, 0.0),
                    ..default()
                });
            }).id();
            
            commands.entity(data.part_entity).add_child(socket_entity);
            selected.0 = Some(socket_entity);
            settings.is_adding = false; // Turn off "plus" mode after spawn
            info!("Spawned and Selected new socket: {:?} ({:?})", socket_entity, data.part_type);
        }
    }
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
    mut selected: ResMut<super::super::ui_inspector::SelectedSocket>,
    mut events: EventReader<bevy_mod_picking::prelude::Pointer<bevy_mod_picking::prelude::Click>>,
    socket_query: Query<Entity, With<super::super::ActorSocket>>,
) {
    for event in events.read() {
        info!("3D Click Target: {:?}", event.target);
        if socket_query.get(event.target).is_ok() {
            selected.0 = Some(event.target);
            info!("Successfully Selected Socket: {:?}", event.target);
        }
    }
}

fn rand_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_millis();
    format!("{:x}", since_the_epoch).chars().rev().take(4).collect()
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
