use bevy::prelude::*;
use client_core::{Project, Selection, DirtyTiles, CommandHistory, RoomTransition, Room, TileType};
use crate::{EditorState, OrbitCamera, BoxGizmos, TILE_SIZE, TILE_H};

pub fn selection_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<Selection>,
    mut project: ResMut<Project>,
    mut history: ResMut<CommandHistory>,
    mut dirty: ResMut<DirtyTiles>,
    mut editor_state: ResMut<EditorState>,
    camera_query: Query<&Transform, With<OrbitCamera>>,
) {
    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) { return; }
    
    let mut move_dx = 0i32;
    let mut move_dz = 0i32;

    if let Ok(cam_transform) = camera_query.get_single() {
        let forward = cam_transform.forward();
        let forward_h = Vec2::new(forward.x, forward.z).normalize_or_zero();
        
        let (f_dx, f_dz) = if forward_h.x.abs() > forward_h.y.abs() {
            if forward_h.x > 0.0 { (1, 0) } else { (-1, 0) }
        } else {
            if forward_h.y > 0.0 { (0, 1) } else { (0, -1) }
        };

        if keyboard.just_pressed(KeyCode::ArrowUp)    { move_dx = f_dx; move_dz = f_dz; }
        if keyboard.just_pressed(KeyCode::ArrowDown)  { move_dx = -f_dx; move_dz = -f_dz; }
        if keyboard.just_pressed(KeyCode::ArrowRight) { move_dx = -f_dz; move_dz = f_dx; }
        if keyboard.just_pressed(KeyCode::ArrowLeft)  { move_dx = f_dz; move_dz = -f_dx; }
    } else {
        if keyboard.just_pressed(KeyCode::ArrowUp)    { move_dz = -1; }
        if keyboard.just_pressed(KeyCode::ArrowDown)  { move_dz = 1; }
        if keyboard.just_pressed(KeyCode::ArrowLeft)  { move_dx = -1; }
        if keyboard.just_pressed(KeyCode::ArrowRight) { move_dx = 1; }
    }

    if move_dx != 0 || move_dz != 0 {
        let room_idx = project.current_room_idx;
        editor_state.last_selected_cell = project.rooms[room_idx].cells[selection.x][selection.z];
        
        let new_x = (selection.x as i32 + move_dx).clamp(0, 15);
        let new_z = (selection.z as i32 + move_dz).clamp(0, 15);
        selection.x = new_x as usize;
        selection.z = new_z as usize;
    }
    
    let room_idx = project.current_room_idx;
    
    if keyboard.just_pressed(KeyCode::KeyF) {
        let cell = editor_state.last_selected_cell;
        history.push_undo(&project);
        project.rooms[room_idx].cells[selection.x][selection.z] = cell;
        mark_tile_dirty(selection.x, selection.z, &mut dirty);
    }

    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::KeyA) {
        history.push_undo(&project);
        let cell = &mut project.rooms[room_idx].cells[selection.x][selection.z];
        if keyboard.just_pressed(KeyCode::KeyQ) { 
            cell.h += 1; 
            if cell.h == 0 { cell.tt = TileType::Empty; }
            else if cell.h > 0 { cell.tt = editor_state.current_type; }
            mark_tile_dirty(selection.x, selection.z, &mut dirty);
        }
        if keyboard.just_pressed(KeyCode::KeyA) { 
            cell.h = (cell.h - 1).max(-1); 
            if cell.h < 0 { cell.tt = TileType::Empty; }
            mark_tile_dirty(selection.x, selection.z, &mut dirty);
        }
    }
}

pub fn mouse_selection_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<OrbitCamera>, Without<crate::SelectionHighlight>)>,
    project: Res<Project>,
    mut selection: ResMut<Selection>,
    mut editor_state: ResMut<EditorState>,
    interaction_query: Query<&Interaction>,
) {
    if !buttons.just_pressed(MouseButton::Left) { return; }
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::None { return; }
    }
    let Ok(window) = windows.get_single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let mut best_hit: Option<(usize, usize, f32)> = None;
            let room = &project.rooms[project.current_room_idx];

            for x in 0..16 {
                for z in 0..16 {
                    let cell = room.cells[x][z];
                    if cell.h >= 0 {
                        let (y_min, y_max) = if cell.h == 0 { (-0.5, 0.0) } else { (0.0, cell.h as f32 * TILE_H) };
                        let min = Vec3::new(x as f32 - 0.5, y_min, z as f32 - 0.5);
                        let max = Vec3::new(x as f32 + 0.5, y_max, z as f32 + 0.5);
                        if let Some(hit_t) = ray_aabb(ray, min, max) {
                            if best_hit.is_none() || hit_t < best_hit.unwrap().2 { best_hit = Some((x, z, hit_t)); }
                        }
                    }
                    let f_min = Vec3::new(x as f32 - 0.5, -0.5, z as f32 - 0.5);
                    let f_max = Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5);
                    if let Some(hit_t) = ray_aabb(ray, f_min, f_max) {
                        if best_hit.is_none() || hit_t < best_hit.unwrap().2 { best_hit = Some((x, z, hit_t)); }
                    }
                }
            }

            if let Some((x, z, _)) = best_hit {
                if selection.x != x || selection.z != z {
                    let room_idx = project.current_room_idx;
                    editor_state.last_selected_cell = project.rooms[room_idx].cells[selection.x][selection.z];
                    selection.x = x;
                    selection.z = z;
                }
            }
        }
    }
}

pub fn room_switching_system(
    keyboard: Res<ButtonInput<KeyCode>>, 
    mut project: ResMut<Project>, 
    mut history: ResMut<CommandHistory>,
    mut transition: ResMut<RoomTransition>,
) {
    if keyboard.just_pressed(KeyCode::BracketRight) {
        history.push_undo(&project);
        let next_idx = project.current_room_idx + 1;
        if next_idx >= project.rooms.len() { project.rooms.push(Room::default()); }
        transition.start(next_idx);
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        if project.current_room_idx > 0 { 
            history.push_undo(&project);
            transition.start(project.current_room_idx - 1);
        }
    }
}

pub fn auto_save_system(project: Res<Project>) {
    if project.is_changed() {
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(json) = serde_json::to_string_pretty(&*project) {
            let _ = std::fs::write("assets/map.json", json);
        }
    }
}

pub fn undo_redo_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut project: ResMut<Project>,
    mut history: ResMut<CommandHistory>,
    mut dirty: ResMut<DirtyTiles>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && keyboard.just_pressed(KeyCode::KeyZ) {
        if let Some(prev) = history.undo(&project) {
            *project = prev;
            dirty.full_rebuild = true;
        }
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyU) {
        if let Some(next) = history.redo(&project) {
            *project = next;
            dirty.full_rebuild = true;
        }
    }
}

pub fn selection_highlight_system(
    selection: Res<Selection>, 
    project: Res<Project>,
    editor_state: Res<EditorState>,
    camera_query: Query<&Transform, With<OrbitCamera>>,
    time: Res<Time>,
    mut gizmos: Gizmos<BoxGizmos>,
) {
    let room_idx = project.current_room_idx;
    let cell = project.rooms[room_idx].cells[selection.x][selection.z];
    let cam_pos = camera_query.get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);
    let is_on = (time.elapsed_seconds() * 5.0).sin() > 0.0;
    if !is_on { return; }

    if cell.h > 0 {
        let h_s = TILE_SIZE * 0.5;
        let base_y = 0.0;
        let top_y = (cell.h as f32 - 1.0) * TILE_H;
        for dx in [-h_s, h_s] {
            for dz in [-h_s, h_s] {
                let start = Vec3::new(selection.x as f32 + dx, base_y, selection.z as f32 + dz);
                let end = Vec3::new(selection.x as f32 + dx, top_y, selection.z as f32 + dz);
                draw_dashed_line(&mut gizmos, start, end, Color::srgba(1.0, 1.0, 1.0, 0.15));
            }
        }
    }

    let top_pos = Vec3::new(selection.x as f32, (cell.h as f32 - 0.5) * TILE_H, selection.z as f32);
    let transform = Transform::from_translation(top_pos).with_scale(Vec3::new(TILE_SIZE * 1.01, TILE_H * 1.01, TILE_SIZE * 1.01));
    let preview_type = editor_state.current_type;
    let color = Color::srgba(1.0, 1.0, 1.0, 0.4);

    match preview_type {
        TileType::Cube => draw_refined_cuboid(&mut gizmos, transform, cam_pos, color),
        _ => {
            let rot = match preview_type {
                TileType::WedgeN => 0.0,
                TileType::WedgeE => -std::f32::consts::FRAC_PI_2,
                TileType::WedgeS => std::f32::consts::PI,
                TileType::WedgeW => std::f32::consts::FRAC_PI_2,
                _ => 0.0,
            };
            draw_refined_wedge(&mut gizmos, transform, rot, cam_pos, color);
        }
    }
}

// Helpers
fn ray_aabb(ray: Ray3d, min: Vec3, max: Vec3) -> Option<f32> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;
    for i in 0..3 {
        let inv_dir = 1.0 / ray.direction[i];
        let mut t1 = (min[i] - ray.origin[i]) * inv_dir;
        let mut t2 = (max[i] - ray.origin[i]) * inv_dir;
        if inv_dir < 0.0 { std::mem::swap(&mut t1, &mut t2); }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
    }
    if t_max >= t_min && t_max > 0.0 { Some(if t_min > 0.0 { t_min } else { t_max }) } else { None }
}

fn draw_refined_cuboid(gizmos: &mut Gizmos<BoxGizmos>, transform: Transform, cam_pos: Vec3, color: Color) {
    let center = transform.translation;
    let half_scale = transform.scale * 0.5;
    let mut v = [Vec3::ZERO; 8];
    let mut i = 0;
    for dx in [-1.0, 1.0] {
        for dy in [-1.0, 1.0] {
            for dz in [-1.0, 1.0] {
                v[i] = center + Vec3::new(dx * half_scale.x, dy * half_scale.y, dz * half_scale.z);
                i += 1;
            }
        }
    }
    let edges = [(0, 1, 1, 3), (2, 3, 1, 2), (4, 5, 0, 3), (6, 7, 0, 2), (0, 2, 1, 5), (1, 3, 1, 4), (4, 6, 0, 5), (5, 7, 0, 4), (0, 4, 3, 5), (1, 5, 3, 4), (2, 6, 2, 5), (3, 7, 2, 4)];
    let face_normals = [Vec3::X, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Y, Vec3::Z, Vec3::NEG_Z];
    for (i1, i2, n1, n2) in edges {
        let is_back1 = face_normals[n1].dot(cam_pos - center) < 0.0;
        let is_back2 = face_normals[n2].dot(cam_pos - center) < 0.0;
        if is_back1 && is_back2 { draw_dashed_line(gizmos, v[i1], v[i2], color.with_alpha(0.15)); } else { gizmos.line(v[i1], v[i2], color); }
    }
}

fn draw_refined_wedge(gizmos: &mut Gizmos<BoxGizmos>, transform: Transform, rot: f32, cam_pos: Vec3, color: Color) {
    let center = transform.translation;
    let half_scale = transform.scale * 0.5;
    let rotation = Quat::from_rotation_y(rot);
    let local_v = [Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, -1.0, -1.0), Vec3::new(-1.0, -1.0, 1.0), Vec3::new(1.0, -1.0, 1.0), Vec3::new(-1.0, 1.0, -1.0), Vec3::new(1.0, 1.0, -1.0)];
    let mut v = [Vec3::ZERO; 6];
    for i in 0..6 { v[i] = center + rotation * (local_v[i] * half_scale); }
    let edges = [(0, 1), (1, 3), (3, 2), (2, 0), (4, 5), (0, 4), (1, 5), (2, 4), (3, 5)];
    for (i1, i2) in edges {
        let edge_center = (v[i1] + v[i2]) * 0.5;
        let is_back = (edge_center - center).dot(cam_pos - center) < -0.2;
        if is_back { draw_dashed_line(gizmos, v[i1], v[i2], color.with_alpha(0.15)); } else { gizmos.line(v[i1], v[i2], color); }
    }
}

fn draw_dashed_line(gizmos: &mut Gizmos<BoxGizmos>, start: Vec3, end: Vec3, color: Color) {
    let dir = end - start;
    let length = dir.length();
    if length < 0.01 { return; }
    let norm = dir / length;
    let dash_len = 0.05;
    let gap_len = 0.05;
    let mut current = 0.0;
    while current < length {
        let dash_end = (current + dash_len).min(length);
        gizmos.line(start + norm * current, start + norm * dash_end, color);
        current += dash_len + gap_len;
    }
}

pub fn mark_tile_dirty(x: usize, z: usize, dirty: &mut DirtyTiles) {
    dirty.tiles.push((x, z));
    if x > 0 { dirty.tiles.push((x - 1, z)); }
    if x < 15 { dirty.tiles.push((x + 1, z)); }
    if z > 0 { dirty.tiles.push((x, z - 1)); }
    if z < 15 { dirty.tiles.push((x, z + 1)); }
}
