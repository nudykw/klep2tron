use bevy::prelude::*;
use super::super::{LassoState, LassoSelectionMode, EditorMode, SlicingSettings, SelectedTriangles, ActorPart, MainEditorCamera};

pub fn lasso_input_system(
    mut lasso_state: ResMut<LassoState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    editor_mode: Res<EditorMode>,
    slicing_settings: Res<SlicingSettings>,
    mut part_query: Query<&mut SelectedTriangles>,
    ui_query: Query<&Interaction, With<Node>>,
) {
    if *editor_mode != EditorMode::Slicing || !slicing_settings.manual_mode {
        if lasso_state.is_active {
            lasso_state.is_active = false;
            lasso_state.points.clear();
        }
        return;
    }

    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    let mut hovering_ui = false;
    for interaction in ui_query.iter() {
        if *interaction != Interaction::None {
            hovering_ui = true;
            break;
        }
    }

    let alt = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    
    if mouse_button.just_pressed(MouseButton::Left) {
        if !hovering_ui {
            lasso_state.is_active = true;
            lasso_state.points.clear();
            lasso_state.points.push(cursor_pos);
            lasso_state.mode = if alt { LassoSelectionMode::Subtract } else { LassoSelectionMode::Add };
        }
    } else if mouse_button.pressed(MouseButton::Left) && lasso_state.is_active {
        if let Some(last) = lasso_state.points.last() {
            if last.distance(cursor_pos) > 4.0 {
                lasso_state.points.push(cursor_pos);
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        lasso_state.is_active = false;
        lasso_state.points.clear();
        for mut selected in part_query.iter_mut() {
            selected.indices.clear();
        }
    }
}

pub fn lasso_render_system(
    lasso_state: Res<LassoState>,
    mut gizmos: Gizmos,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainEditorCamera>>,
) {
    if !lasso_state.is_active || lasso_state.points.len() < 2 { return; }
    
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };
    
    let color = if lasso_state.mode == LassoSelectionMode::Add {
        Color::srgb(0.0, 1.0, 1.0) // Cyan
    } else {
        Color::srgb(1.0, 0.2, 0.2) // Red
    };

    // Draw in screen space by projecting rays at near plane
    let draw_at_dist = 0.1;
    let mut world_points = Vec::new();
    for p in &lasso_state.points {
        if let Some(ray) = camera.viewport_to_world(camera_transform, *p) {
            world_points.push(ray.origin + ray.direction * draw_at_dist);
        }
    }

    if world_points.len() < 2 { return; }

    for i in 0..world_points.len() - 1 {
        gizmos.line(world_points[i], world_points[i+1], color);
    }
    
    // Close the loop
    if world_points.len() > 2 {
        gizmos.line(*world_points.last().unwrap(), world_points[0], color);
    }
}

pub fn triangle_selection_system(
    mut lasso_state: ResMut<LassoState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainEditorCamera>>,
    mut part_query: Query<(Entity, &ActorPart, &Handle<Mesh>, &GlobalTransform, &mut SelectedTriangles)>,
    meshes: Res<Assets<Mesh>>,
) {
    if !lasso_state.is_active || !mouse_button.just_released(MouseButton::Left) { return; }
    
    // Handle "Reset" or "Single Click Selection" (Click on empty space or tiny lasso)
    if lasso_state.points.len() < 5 {
        let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };
        let last_pos = *lasso_state.points.last().unwrap();
        let ray = camera.viewport_to_world(camera_transform, last_pos).unwrap();
        
        let mut hit_anything = false;
        let mut closest_hit_dist = f32::MAX;
        let mut closest_hit_entity = None;
        let mut closest_hit_index = None;

        for (entity, _, mesh_handle, transform, _) in part_query.iter() {
            if let Some(mesh) = meshes.get(mesh_handle) {
                if let Some(hit) = crate::actor_editor::geometry::raycast::ray_mesh_intersection(ray.origin, ray.direction.into(), mesh, transform) {
                    hit_anything = true;
                    if hit.distance < closest_hit_dist {
                        closest_hit_dist = hit.distance;
                        closest_hit_entity = Some(entity);
                        closest_hit_index = Some(hit.triangle_index);
                    }
                }
            }
        }
        
        if hit_anything {
            if let (Some(entity), Some(index)) = (closest_hit_entity, closest_hit_index) {
                if let Ok((_, _, _, _, mut selected)) = part_query.get_mut(entity) {
                    if lasso_state.mode == LassoSelectionMode::Add {
                        selected.indices.insert(index);
                    } else {
                        selected.indices.remove(&index);
                    }
                }
            }
        } else {
            // Clear selection if clicking on empty space
            for (_, _, _, _, mut selected) in part_query.iter_mut() {
                selected.indices.clear();
            }
        }
        
        lasso_state.is_active = false;
        lasso_state.points.clear();
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };
    
    let mut selected_per_part = Vec::new();

    for (entity, part, mesh_handle, transform, _) in part_query.iter() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            let mut newly_selected = std::collections::HashSet::new();
            
            let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap();
            let indices = mesh.indices().unwrap();
            
            let triangles = match indices {
                bevy::render::mesh::Indices::U16(vec) => vec.chunks(3).map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect::<Vec<_>>(),
                bevy::render::mesh::Indices::U32(vec) => vec.chunks(3).map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect::<Vec<_>>(),
            };

            for (tri_idx, tri) in triangles.iter().enumerate() {
                let mut all_inside = true;
                for &v_idx in tri {
                    let local_pos = Vec3::from(positions[v_idx]);
                    let world_pos = transform.transform_point(local_pos);
                    if let Some(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) {
                        if !point_in_polygon(screen_pos, &lasso_state.points) {
                            all_inside = false;
                            break;
                        }
                    } else {
                        all_inside = false;
                        break;
                    }
                }

                if all_inside {
                    let v0 = transform.transform_point(Vec3::from(positions[tri[0]]));
                    let v1 = transform.transform_point(Vec3::from(positions[tri[1]]));
                    let v2 = transform.transform_point(Vec3::from(positions[tri[2]]));
                    let normal = (v1 - v0).cross(v2 - v0).normalize();
                    let world_center = (v0 + v1 + v2) / 3.0;
                    
                    let dir_to_cam = (camera_transform.translation() - world_center).normalize();
                    
                    // Backface culling: only check visibility for front-facing triangles
                    if normal.dot(dir_to_cam) > 0.0 {
                        if is_visible(world_center, camera_transform.translation(), &part_query, &meshes) {
                            newly_selected.insert(tri_idx);
                        }
                    }
                }
            }
            
            if !newly_selected.is_empty() {
                selected_per_part.push((entity, *part, newly_selected));
            }
        }
    }

    // Apply Hierarchy Filter
    if !selected_per_part.is_empty() {
        let mut best_part = ActorPart::Engine;
        for (_, part, _) in &selected_per_part {
             match (part, best_part) {
                 (ActorPart::Head, _) => best_part = ActorPart::Head,
                 (ActorPart::Body, ActorPart::Head) => {},
                 (ActorPart::Body, _) => best_part = ActorPart::Body,
                 _ => {}
             }
        }
        selected_per_part.retain(|(_, part, _)| *part == best_part);
    }

    // Update Components
    for (entity, _, newly_selected) in selected_per_part {
        if let Ok((_, _, _, _, mut selected)) = part_query.get_mut(entity) {
            if lasso_state.mode == LassoSelectionMode::Add {
                for idx in newly_selected { selected.indices.insert(idx); }
            } else {
                for idx in newly_selected { selected.indices.remove(&idx); }
            }
        }
    }

    lasso_state.is_active = false;
    lasso_state.points.clear();
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        if ((polygon[i].y > point.y) != (polygon[j].y > point.y)) &&
           (point.x < (polygon[j].x - polygon[i].x) * (point.y - polygon[i].y) / (polygon[j].y - polygon[i].y) + polygon[i].x) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn is_visible(
    target: Vec3,
    camera_pos: Vec3,
    part_query: &Query<(Entity, &ActorPart, &Handle<Mesh>, &GlobalTransform, &mut SelectedTriangles)>,
    meshes: &Res<Assets<Mesh>>,
) -> bool {
    let dir = (target - camera_pos).normalize();
    let dist_to_target = camera_pos.distance(target);
    
    for (_, _, mesh_handle, transform, _) in part_query.iter() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            if let Some(hit) = crate::actor_editor::geometry::raycast::ray_mesh_intersection(camera_pos, dir.into(), mesh, transform) {
                if hit.distance < dist_to_target - 0.001 {
                    return false;
                }
            }
        }
    }
    true
}
