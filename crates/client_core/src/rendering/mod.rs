use bevy::prelude::*;
use crate::{Project, ClientAssets, TileMap, DirtyTiles, TileType, TileEntity};

#[derive(Resource, Default)]
pub struct MaterialCache {
    pub map: std::collections::HashMap<(i32, bool), (Handle<StandardMaterial>, Handle<StandardMaterial>)>,
}

pub fn map_rendering_system(
    mut commands: Commands,
    project: Res<Project>,
    assets: Res<ClientAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _tile_query: Query<Entity, With<TileEntity>>,
    mut cache: Local<MaterialCache>,
    mut first_run: Local<bool>,
    mut last_room: Local<Option<usize>>,
    mut tile_map: ResMut<TileMap>,
    mut dirty: ResMut<DirtyTiles>,
) {
    if assets.cube_mesh == Handle::default() { return; } 
    let room_changed = last_room.map_or(true, |r| r != project.current_room_idx);
    
    if !*first_run || room_changed || dirty.full_rebuild || project.is_changed() {
        if !*first_run || room_changed || dirty.full_rebuild {
            *first_run = true;
            *last_room = Some(project.current_room_idx);
            dirty.full_rebuild = false;
            dirty.tiles.clear();
            
            for entity in _tile_query.iter() {
                if let Some(ec) = commands.get_entity(entity) {
                    ec.despawn_recursive();
                }
            }
            tile_map.entities.clear();
            for x in 0..16 { for z in 0..16 { dirty.tiles.push((x, z)); } }
        } else if project.is_changed() {
            for x in 0..16 { for z in 0..16 { dirty.tiles.push((x, z)); } }
        }
    } else if dirty.tiles.is_empty() {
        return;
    }
    
    if project.rooms.is_empty() { return; }
    let room = &project.rooms[project.current_room_idx];
    
    let tiles_to_process = std::mem::take(&mut dirty.tiles);
    for (x, z) in tiles_to_process {
        if let Some(entities) = tile_map.entities.remove(&(x, z)) {
            for entity in entities { 
                if let Some(ec) = commands.get_entity(entity) {
                    ec.despawn_recursive();
                }
            }
        }
        
        let cell = room.cells[x][z];
        if cell.h < 0 { continue; }
        let is_even = (x + z) % 2 == 0;

        let mut tx = x as i32;
        let mut tz = z as i32;
        let mut steps = 0;
        let mut found_cube_h = None;
        let initial_tt = cell.tt;
        
        while steps < 16 {
            let (dx, dz) = match initial_tt {
                TileType::WedgeN => (0, -1), TileType::WedgeE => (1, 0),
                TileType::WedgeS => (0, 1), TileType::WedgeW => (-1, 0),
                _ => break,
            };
            tx += dx; tz += dz;
            if tx < 0 || tx >= 16 || tz < 0 || tz >= 16 { break; }
            
            let c = room.cells[tx as usize][tz as usize];
            if c.tt == initial_tt {
                found_cube_h = Some(c.h);
                steps += 1;
                continue;
            } else {
                break;
            }
        }
        let target_h = found_cube_h.unwrap_or(cell.h);

        let (mat_top, mat_side) = cache.map.entry((target_h, is_even)).or_insert_with(|| {
            let base_color = get_rainbow_color(target_h, is_even);
            let side_color = Color::from(LinearRgba::from(base_color) * 0.7);
            (materials.add(StandardMaterial { base_color, ..default() }), materials.add(StandardMaterial { base_color: side_color, ..default() }))
        }).clone();

        let (mesh, rot) = match cell.tt {
            TileType::Cube => (assets.cube_mesh.clone(), 0.0),
            TileType::WedgeN => (assets.wedge_mesh.clone(), 0.0),
            TileType::WedgeE => (assets.wedge_mesh.clone(), -std::f32::consts::FRAC_PI_2),
            TileType::WedgeS => (assets.wedge_mesh.clone(), std::f32::consts::PI),
            TileType::WedgeW => (assets.wedge_mesh.clone(), std::f32::consts::FRAC_PI_2),
            _ => (assets.cube_mesh.clone(), 0.0),
        };

        let h_val = cell.h as f32;
        let total_height = h_val * 0.5;
        let mut entities = Vec::new();
        
        let top_id = commands.spawn((PbrBundle {
            mesh: mesh.clone(), 
            material: mat_top,
            transform: Transform::from_translation(Vec3::new(x as f32, total_height - 0.25, z as f32))
                .with_scale(Vec3::new(1.0, 0.5, 1.0)).with_rotation(Quat::from_rotation_y(rot)),
            ..default()
        }, TileEntity)).id();
        entities.push(top_id);

        let foundation_bottom = -0.5;
        let column_top = total_height - 0.5;
        let column_h = column_top - foundation_bottom;
        
        if column_h > 0.01 {
            let col_id = commands.spawn((PbrBundle {
                mesh: assets.cube_mesh.clone(), 
                material: mat_side.clone(),
                transform: Transform::from_translation(Vec3::new(x as f32, foundation_bottom + column_h * 0.5, z as f32))
                    .with_scale(Vec3::new(1.0, column_h, 1.0)),
                ..default()
            }, TileEntity)).id();
            entities.push(col_id);
        }
        
        tile_map.entities.insert((x, z), entities);
    }
}

pub fn get_rainbow_color(h: i32, is_even: bool) -> Color {
    let rainbow = [
        Color::srgb(1.0, 0.2, 0.2), 
        Color::srgb(1.0, 0.5, 0.0), 
        Color::srgb(1.0, 0.9, 0.0), 
        Color::srgb(0.2, 0.8, 0.2), 
        Color::srgb(0.0, 0.6, 1.0), 
        Color::srgb(0.3, 0.3, 0.9), 
        Color::srgb(0.6, 0.2, 0.8), 
    ];
    let idx = (h.max(0) as usize) % rainbow.len();
    let mut color = LinearRgba::from(rainbow[idx]);
    if !is_even {
        color.red *= 0.9;
        color.green *= 0.9;
        color.blue *= 0.9;
    }
    Color::from(color)
}
