use bevy::prelude::*;
use super::{GizmoAxis, GizmoAxisType, GizmoAction, ManualGizmoInteraction, SocketGizmo};

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

pub fn ray_segment_closest_points(ray: &Ray3d, p1: Vec3, p2: Vec3) -> Option<(f32, f32)> {
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

pub fn actor_part_picking_priority_system(
    selected: Res<crate::actor_editor::ui::inspector::SelectedSocket>,
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
