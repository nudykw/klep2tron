//! Static colliders generated from the tile map.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::rendering::tile_geometry;
use crate::{Cell, Project};

/// Marks a collider generated from the tile map.
#[derive(Component)]
pub struct TileCollider;

/// Room index the current colliders were built for (`None` = nothing built).
#[derive(Resource, Default)]
pub struct TileColliderRoom(pub Option<usize>);

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

    for x in 0..16 {
        for z in 0..16 {
            let cell = room.cells[x][z];
            if cell.h < 0 {
                continue;
            }
            spawn_cell_collider(&mut commands, x, z, &cell);
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

fn spawn_cell_collider(commands: &mut Commands, x: usize, z: usize, cell: &Cell) {
    if tile_geometry::is_wedge(cell.tt) {
        if let Some(column) = tile_geometry::column_placement(x, z, cell) {
            spawn_box(commands, column.translation, column.scale);
        }
        // Exact hull of the slope's rendered vertices.
        let points = tile_geometry::wedge_world_vertices(x, z, cell);
        if let Some(collider) = Collider::convex_hull(points) {
            commands.spawn((RigidBody::Static, collider, TileCollider));
        }
    } else {
        // Cube: one box from the foundation up to the walkable surface.
        let top = cell.h as f32 * 0.5;
        let height = top - tile_geometry::FOUNDATION_BOTTOM;
        let center_y = (top + tile_geometry::FOUNDATION_BOTTOM) * 0.5;
        spawn_box(
            commands,
            Vec3::new(x as f32, center_y, z as f32),
            Vec3::new(1.0, height, 1.0),
        );
    }
}

fn spawn_box(commands: &mut Commands, translation: Vec3, size: Vec3) {
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
        Transform::from_translation(translation),
        TileCollider,
    ));
}
