//! Static colliders generated from the tile map.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::rendering::tile_geometry;
use crate::{Cell, Project, TileType};

/// Marks a collider generated from the tile map.
#[derive(Component)]
pub struct TileCollider;

/// Room index the current colliders were built for (`None` = nothing built).
#[derive(Resource, Default)]
pub struct TileColliderRoom(pub Option<usize>);

/// Placement of a merged cuboid box in world units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MergedBox {
    pub translation: Vec3,
    pub size: Vec3,
}

/// Merges contiguous, coplanar `TileType::Cube` cells of identical height into
/// maximal rectangular boxes.
///
/// Eliminates internal seams between adjacent floor and wall tiles, preventing
/// edge snagging (ghost collisions) while reducing physics collider counts.
pub fn greedy_cube_boxes(cells: &[[Cell; 16]; 16]) -> Vec<MergedBox> {
    let mut visited = [[false; 16]; 16];
    let mut boxes = Vec::new();

    for x in 0..16 {
        for z in 0..16 {
            if visited[x][z] {
                continue;
            }
            let cell = cells[x][z];
            if cell.tt != TileType::Cube || cell.h < 0 {
                continue;
            }
            let target_h = cell.h;

            // 1. Expand along X as far as possible
            let mut x_end = x;
            while x_end + 1 < 16
                && !visited[x_end + 1][z]
                && cells[x_end + 1][z].tt == TileType::Cube
                && cells[x_end + 1][z].h == target_h
            {
                x_end += 1;
            }

            // 2. Expand the [x..=x_end] span along Z as far as possible
            let mut z_end = z;
            while z_end + 1 < 16 {
                let row_matches = (x..=x_end).all(|xi| {
                    !visited[xi][z_end + 1]
                        && cells[xi][z_end + 1].tt == TileType::Cube
                        && cells[xi][z_end + 1].h == target_h
                });
                if row_matches {
                    z_end += 1;
                } else {
                    break;
                }
            }

            // 3. Mark all merged cells as visited
            for row in visited[x..=x_end].iter_mut() {
                row[z..=z_end].fill(true);
            }

            let width = (x_end - x + 1) as f32;
            let depth = (z_end - z + 1) as f32;
            let center_x = (x as f32 + x_end as f32) * 0.5;
            let center_z = (z as f32 + z_end as f32) * 0.5;

            let top = target_h as f32 * 0.5;
            let height = top - tile_geometry::FOUNDATION_BOTTOM;
            let center_y = (top + tile_geometry::FOUNDATION_BOTTOM) * 0.5;

            boxes.push(MergedBox {
                translation: Vec3::new(center_x, center_y, center_z),
                size: Vec3::new(width, height, depth),
            });
        }
    }

    boxes
}

/// Rebuilds all tile colliders when the room changes or the map is mutated.
///
/// The full path is cheap enough for a 16x16 room and keeps collision always in
/// sync with [`crate::rendering::map_rendering_system`].
pub fn rebuild_tile_colliders(
    mut commands: Commands,
    project: Res<Project>,
    existing: Query<Entity, With<TileCollider>>,
    mut built: ResMut<TileColliderRoom>,
) {
    let room_changed = built.0 != Some(project.current_room_idx);
    if !room_changed && !project.is_changed() {
        return;
    }
    built.0 = Some(project.current_room_idx);

    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    let Some(room) = project.rooms.get(project.current_room_idx) else {
        return;
    };

    // 1. Greedy-merged cube colliders (eliminates seams and internal edges)
    for b in greedy_cube_boxes(&room.cells) {
        spawn_box(&mut commands, b.translation, b.size);
    }

    // 2. Slope colliders (wedges)
    for x in 0..16 {
        for z in 0..16 {
            let cell = room.cells[x][z];
            if cell.h < 0 {
                continue;
            }
            if tile_geometry::is_wedge(cell.tt) {
                if let Some(column) = tile_geometry::column_placement(x, z, &cell) {
                    spawn_box(&mut commands, column.translation, column.scale);
                }
                let points = tile_geometry::wedge_world_vertices(x, z, &cell);
                if let Some(collider) = Collider::convex_hull(points) {
                    commands.spawn((RigidBody::Static, collider, TileCollider));
                }
            }
        }
    }
}

/// Despawns tile colliders and clears the cache (used when leaving the game).
pub fn clear_tile_colliders(
    mut commands: Commands,
    existing: Query<Entity, With<TileCollider>>,
    mut built: ResMut<TileColliderRoom>,
) {
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }
    built.0 = None;
}

fn spawn_box(commands: &mut Commands, translation: Vec3, size: Vec3) {
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
        Transform::from_translation(translation),
        TileCollider,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_cells() -> [[Cell; 16]; 16] {
        [[Cell { h: -1, tt: TileType::Empty }; 16]; 16]
    }

    #[test]
    fn single_cube_produces_one_box() {
        let mut cells = empty_cells();
        cells[3][4] = Cell { h: 2, tt: TileType::Cube };

        let boxes = greedy_cube_boxes(&cells);
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].size, Vec3::new(1.0, 1.5, 1.0));
        assert_eq!(boxes[0].translation, Vec3::new(3.0, 0.25, 4.0));
    }

    #[test]
    fn uniform_16x16_room_merges_into_single_collider() {
        let cells = [[Cell { h: 1, tt: TileType::Cube }; 16]; 16];
        let boxes = greedy_cube_boxes(&cells);

        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].size, Vec3::new(16.0, 1.0, 16.0));
        assert_eq!(boxes[0].translation, Vec3::new(7.5, 0.0, 7.5));
    }

    #[test]
    fn different_heights_form_separate_boxes() {
        let mut cells = empty_cells();
        // 2x2 of h=1
        for row in &mut cells[..2] {
            for cell in &mut row[..2] {
                *cell = Cell { h: 1, tt: TileType::Cube };
            }
        }
        // Adjacent 2x2 of h=3
        for row in &mut cells[2..4] {
            for cell in &mut row[..2] {
                *cell = Cell { h: 3, tt: TileType::Cube };
            }
        }

        let boxes = greedy_cube_boxes(&cells);
        assert_eq!(boxes.len(), 2);
        assert!(boxes.iter().any(|b| b.size.x == 2.0 && b.size.z == 2.0 && b.size.y == 1.0));
        assert!(boxes.iter().any(|b| b.size.x == 2.0 && b.size.z == 2.0 && b.size.y == 2.0));
    }

    #[test]
    fn wedges_are_ignored_by_greedy_cube_boxes() {
        let mut cells = empty_cells();
        cells[5][5] = Cell { h: 2, tt: TileType::WedgeN };

        let boxes = greedy_cube_boxes(&cells);
        assert!(boxes.is_empty());
    }
}
