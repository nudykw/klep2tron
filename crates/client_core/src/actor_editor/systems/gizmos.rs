use bevy::prelude::*;
use super::super::{ActorSocket, ui_inspector::SelectedSocket};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxisType {
    X, Y, Z
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAction {
    Translate,
    Rotate,
}

#[derive(Component)]
pub struct SocketGizmo;

#[derive(Component)]
pub struct GizmoAxis {
    pub axis: GizmoAxisType,
    pub action: GizmoAction,
}

pub fn update_socket_gizmos_system(
    mut commands: Commands,
    selected: Res<SelectedSocket>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gizmo_query: Query<Entity, With<SocketGizmo>>,
    socket_query: Query<Entity, With<ActorSocket>>,
) {
    let selected_entity = selected.0;
    
    // Log selection state for debugging
    if selected.is_changed() {
        info!("SelectedSocket changed to: {:?}", selected_entity);
    }

    let mut needs_spawn = false;
    
    // Check if we need to despawn current gizmo
    for entity in gizmo_query.iter() {
        // If nothing selected or selection changed, despawn
        if selected_entity.is_none() || selected.is_changed() {
            commands.entity(entity).despawn_recursive();
            needs_spawn = selected_entity.is_some();
        }
    }

    // If nothing spawned yet but something is selected, spawn
    if gizmo_query.iter().count() == 0 && selected_entity.is_some() {
        needs_spawn = true;
    }

    if needs_spawn {
        if let Some(target) = selected_entity {
            if let Ok(_socket) = socket_query.get(target) {
                info!("Spawning Gizmo for socket: {:?}", target);
                // Spawn a new gizmo at root (decoupled)
                commands.spawn((
                    SpatialBundle::default(),
                    SocketGizmo,
                    SocketLink(target),
                    bevy_mod_picking::prelude::Pickable::default(),
                    Name::new("SocketGizmo"),
                    crate::actor_editor::ActorEditorEntity,
                )).with_children(|gizmo_root| {
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::X, GizmoAction::Translate, Color::srgb(1.0, 0.2, 0.2), target);
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::Y, GizmoAction::Translate, Color::srgb(0.2, 1.0, 0.2), target);
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::Z, GizmoAction::Translate, Color::srgb(0.2, 0.2, 1.0), target);

                    // Spawning rotation rings
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::X, GizmoAction::Rotate, Color::srgb(1.0, 0.2, 0.2), target);
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::Y, GizmoAction::Rotate, Color::srgb(0.2, 1.0, 0.2), target);
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::Z, GizmoAction::Rotate, Color::srgb(0.2, 0.2, 1.0), target);

                });
            }
        }
    }
}

#[derive(Component)]
pub struct SocketLink(pub Entity);

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

pub fn xray_material_system(
    viewport_settings: Res<super::super::ViewportSettings>,
    inspection_settings: Res<crate::actor_editor::InspectionSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    part_query: Query<(Entity, &Handle<StandardMaterial>), With<crate::actor_editor::ActorPart>>,
    mut last_xray: Local<bool>,
) {
    // If inspection is active, don't interfere with its transparency/highlighting logic
    if inspection_settings.is_active {
        return;
    }

    let current_xray = viewport_settings.xray;
    *last_xray = current_xray;

    for (entity, mat_handle) in part_query.iter() {
        if let Some(material) = materials.get_mut(mat_handle) {
            let target_alpha = if current_xray { 0.1 } else { 1.0 };
            let target_mode = if current_xray { AlphaMode::Blend } else { AlphaMode::Opaque };

            // Only update if different
            if (material.base_color.alpha() - target_alpha).abs() > 0.01 || material.alpha_mode != target_mode {
                info!("Applying X-Ray to {:?}: a={}, mode={:?}", entity, target_alpha, target_mode);
                
                let mut color = material.base_color.to_srgba();
                color.alpha = target_alpha;
                
                material.base_color = Color::Srgba(color);
                material.alpha_mode = target_mode;
            }
        }
    }
}


fn spawn_axis(
    parent: &mut ChildBuilder,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    axis: GizmoAxisType,
    action: GizmoAction,
    color: Color,
    target: Entity,
) {
    match action {
        GizmoAction::Translate => {
            let rotation = match axis {
                GizmoAxisType::X => Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                GizmoAxisType::Y => Quat::IDENTITY,
                GizmoAxisType::Z => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            };

            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(0.015, 1.0)),
                    material: materials.add(StandardMaterial {
                        base_color: color,
                        ..default()
                    }),
                    transform: Transform::from_rotation(rotation).with_translation(rotation * Vec3::Y * 0.5),
                    ..default()
                },
                GizmoAxis { axis, action },
                ManualGizmoInteraction::default(),
                SocketLink(target),
                bevy_mod_picking::prelude::PickableBundle::default(),
            )).with_children(|axis_p| {
                // Cone tip
                axis_p.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cone { radius: 0.05, height: 0.15 }),
                        material: materials.add(StandardMaterial { 
                            base_color: color,
                            ..default() 
                        }),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..default()
                    },
                    GizmoAxis { axis, action },
                    ManualGizmoInteraction::default(),
                    SocketLink(target),
                    bevy_mod_picking::prelude::PickableBundle::default(),
                ));
            });
        },
        GizmoAction::Rotate => {
            let rotation = match axis {
                GizmoAxisType::X => Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                GizmoAxisType::Y => Quat::IDENTITY,
                GizmoAxisType::Z => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            };

            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Torus { minor_radius: 0.01, major_radius: 0.75 }),
                    material: materials.add(StandardMaterial {
                        base_color: color,
                        unlit: true,
                        ..default()
                    }),
                    transform: Transform::from_rotation(rotation),
                    ..default()
                },
                GizmoAxis { axis, action },
                ManualGizmoInteraction::default(),
                SocketLink(target),
                bevy_mod_picking::prelude::PickableBundle::default(),
            ));
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ManualGizmoInteraction {
    #[default]
    None,
    Hovered,
    Pressed,
}
pub fn manual_gizmo_picking_system(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::actor_editor::MainEditorCamera>>,
    mut gizmo_query: Query<(Entity, &GlobalTransform, &mut ManualGizmoInteraction), With<GizmoAxis>>,
    gizmo_axis_query: Query<&GizmoAxis>,
    ui_query: Query<&Interaction, With<Node>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    // Skip if interacting with UI
    for interaction in ui_query.iter() {
        if *interaction != Interaction::None {
            if gizmo_query.iter().any(|(_, _, interaction)| *interaction != ManualGizmoInteraction::None) {
                // Only log if we were actually hovering something
                info!("Gizmo picking BLOCKED by UI Interaction");
            }
            // Reset hover states of all gizmos if we are over UI
            for (_, _, mut interaction) in gizmo_query.iter_mut() {
                *interaction = ManualGizmoInteraction::None;
            }
            return;
        }
    }

    let Ok((camera, camera_gt)) = camera_query.get_single() else { return; };
    let Some(ray) = camera.viewport_to_world(camera_gt, cursor_pos) else { return; };

    // If any gizmo is already pressed (being dragged), don't update hover states for others
    if gizmo_query.iter().any(|(_, _, interaction)| *interaction == ManualGizmoInteraction::Pressed) {
        return;
    }

    let mut closest_gizmo: Option<(Entity, f32)> = None;

    for (entity, gt, _) in gizmo_query.iter() {
        let axis_data = if let Ok(a) = gizmo_axis_query.get(entity) { a } else { continue; };
        let center = gt.translation();
        
        let axis_dir = match axis_data.axis {
            GizmoAxisType::X => Vec3::X,
            GizmoAxisType::Y => Vec3::Y,
            GizmoAxisType::Z => Vec3::Z,
        };

        match axis_data.action {
            GizmoAction::Translate => {
                // Check distance to the line segment [center - 0.1, center + axis_dir * 1.0]
                // We use a bit of "backwards" length for easier picking near the root
                let start = center - axis_dir * 0.1;
                let end = center + axis_dir * 1.0;
                
                // Closest point on segment to ray
                if let Some((t_ray, _t_seg)) = ray_segment_closest_points(&ray, start, end) {
                    let ray_point = ray.origin + Vec3::from(ray.direction) * t_ray;
                    let seg_point = start.lerp(end, _t_seg.clamp(0.0, 1.0));
                    let dist = ray_point.distance(seg_point);
                    
                    if dist < 0.15 {
                        if closest_gizmo.is_none() || dist < closest_gizmo.unwrap().1 {
                            closest_gizmo = Some((entity, dist));
                        }
                    }
                }
            },
            GizmoAction::Rotate => {
                // Check distance to the ring (torus)
                let denom = axis_dir.dot(ray.direction.into());
                if denom.abs() > 0.0001 {
                    let t = (center - ray.origin).dot(axis_dir) / denom;
                    if t > 0.0 {
                        let hit_point = ray.origin + Vec3::from(ray.direction) * t;
                        let dist_to_center = hit_point.distance(center);
                        let dist_to_ring = (dist_to_center - 0.75).abs();
                        
                        if dist_to_ring < 0.1 {
                            if closest_gizmo.is_none() || dist_to_ring < closest_gizmo.unwrap().1 {
                                closest_gizmo = Some((entity, dist_to_ring));
                            }
                        }
                    }
                }
            }
        }
    }

    // Apply interaction
    for (entity, _, mut interaction) in gizmo_query.iter_mut() {
        if let Some((closest_entity, _)) = closest_gizmo {
            if entity == closest_entity {
                if *interaction == ManualGizmoInteraction::None {
                    *interaction = ManualGizmoInteraction::Hovered;
                }
            } else {
                *interaction = ManualGizmoInteraction::None;
            }
        } else {
            *interaction = ManualGizmoInteraction::None;
        }
    }
}

fn ray_segment_closest_points(ray: &Ray3d, p1: Vec3, p2: Vec3) -> Option<(f32, f32)> {
    let u = Vec3::from(ray.direction);
    let v = p2 - p1;
    let w = ray.origin - p1;
    let a = u.dot(u);
    let b = u.dot(v);
    let c = v.dot(v);
    let d = u.dot(w);
    let e = v.dot(w);
    let denom = a * c - b * b;
    
    if denom.abs() < 1e-6 {
        return None;
    }
    
    let t_ray = (b * e - c * d) / denom;
    let t_seg = (a * e - b * d) / denom;
    Some((t_ray, t_seg))
}

pub fn gizmo_highlight_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    axis_query: Query<(&ManualGizmoInteraction, &GizmoAxis, &Handle<StandardMaterial>)>,
    mut busy: ResMut<crate::actor_editor::GizmoBusy>,
) {
    let mut any_hovered = false;
    for (interaction, axis, mat_handle) in axis_query.iter() {
        if let Some(mat) = materials.get_mut(mat_handle) {
            let is_active = *interaction != ManualGizmoInteraction::None;
            if is_active { any_hovered = true; }

            let base_color = match axis.axis {
                GizmoAxisType::X => Color::srgb(1.0, 0.2, 0.2),
                GizmoAxisType::Y => Color::srgb(0.2, 1.0, 0.2),
                GizmoAxisType::Z => Color::srgb(0.2, 0.2, 1.0),
            };

            if is_active {
                let mut bright = base_color.to_srgba();
                bright.red = (bright.red + 0.4).min(1.0);
                bright.green = (bright.green + 0.4).min(1.0);
                bright.blue = (bright.blue + 0.4).min(1.0);
                mat.base_color = Color::Srgba(bright);
            } else {
                mat.base_color = base_color;
            }
        }
    }
    busy.0 = any_hovered;
}


pub fn socket_gizmo_sync_system(
    mut socket_query: Query<(&Transform, &mut ActorSocket)>,
) {
    for (transform, mut socket) in socket_query.iter_mut() {
        socket.definition.position = transform.translation;
        socket.definition.rotation = transform.rotation;
    }
}

pub fn actor_part_picking_priority_system(
    selected: Res<crate::actor_editor::ui_inspector::SelectedSocket>,
    mut pickable_query: Query<(Entity, &mut bevy_mod_picking::prelude::Pickable, Option<&SocketGizmo>, Option<&GizmoAxis>)>,
) {
    let has_selection = selected.0.is_some();
    
    for (_entity, mut pickable, gizmo_opt, axis_opt) in pickable_query.iter_mut() {
        // Gizmos are always pickable
        if gizmo_opt.is_some() || axis_opt.is_some() {
            pickable.is_hoverable = true;
            pickable.should_block_lower = true;
            continue;
        }

        // Everything else:
        if has_selection {
            pickable.is_hoverable = false;
            pickable.should_block_lower = false;
        } else {
            pickable.is_hoverable = true;
            pickable.should_block_lower = true;
        }
    }
}

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
                                
                                transform.translation += move_dir * axis_movement;
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
                                                // Always apply from the START rotation to avoid noise accumulation
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
