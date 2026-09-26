//! Single source of truth for a tile's world placement.
//!
//! Both the renderer (`map_rendering_system`) and the physics colliders
//! (`physics::colliders`) derive transforms from here, so visuals and collision
//! never drift apart.

use bevy::prelude::*;

use crate::{Cell, TileType};

/// Bottom of a tile's foundation column (world units).
pub const FOUNDATION_BOTTOM: f32 = -0.5;
/// Vertical thickness of the top slab / slope, in local units (scaled by 0.5).
pub const SLAB_THICKNESS: f32 = 0.5;

/// Local-space vertices of `assets/3dModels/Room/Bricks/wedge.obj`.
///
/// Low edge at `z = +0.5` (y = -0.5), rising to `z = -0.5` (y = +0.5).
pub const WEDGE_VERTICES: [[f32; 3]; 6] = [
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
];

/// World placement of a tile part.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TilePlacement {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

/// Yaw that orients a wedge toward the direction encoded by [`TileType`].
pub fn wedge_yaw(tile_type: TileType) -> f32 {
    match tile_type {
        TileType::WedgeE => -std::f32::consts::FRAC_PI_2,
        TileType::WedgeS => std::f32::consts::PI,
        TileType::WedgeW => std::f32::consts::FRAC_PI_2,
        _ => 0.0,
    }
}

/// Placement of the top slab (cubes / new-room defaults) or the slope (wedges).
pub fn top_placement(x: usize, z: usize, cell: &Cell) -> TilePlacement {
    let surface = cell.h as f32 * 0.5;
    TilePlacement {
        translation: Vec3::new(x as f32, surface - SLAB_THICKNESS * 0.5, z as f32),
        scale: Vec3::new(1.0, SLAB_THICKNESS, 1.0),
        rotation: Quat::from_rotation_y(wedge_yaw(cell.tt)),
    }
}

/// Placement of the foundation column, or `None` when there is nothing to fill.
pub fn column_placement(x: usize, z: usize, cell: &Cell) -> Option<TilePlacement> {
    let column_top = cell.h as f32 * 0.5 - SLAB_THICKNESS;
    let column_h = column_top - FOUNDATION_BOTTOM;
    if column_h <= 0.01 {
        return None;
    }
    Some(TilePlacement {
        translation: Vec3::new(x as f32, FOUNDATION_BOTTOM + column_h * 0.5, z as f32),
        scale: Vec3::new(1.0, column_h, 1.0),
        rotation: Quat::IDENTITY,
    })
}

/// Whether `tile_type` is one of the four slope orientations.
pub fn is_wedge(tile_type: TileType) -> bool {
    matches!(
        tile_type,
        TileType::WedgeN | TileType::WedgeE | TileType::WedgeS | TileType::WedgeW
    )
}

/// World-space vertices of a wedge cell (for collision hulls).
pub fn wedge_world_vertices(x: usize, z: usize, cell: &Cell) -> Vec<Vec3> {
    let placement = top_placement(x, z, cell);
    WEDGE_VERTICES
        .iter()
        .map(|v| placement.rotation * (Vec3::from_array(*v) * placement.scale) + placement.translation)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(h: i32, tt: TileType) -> Cell {
        Cell { h, tt }
    }

    #[test]
    fn cube_top_surface_matches_half_height() {
        let placement = top_placement(0, 0, &cell(3, TileType::Cube));
        // Surface must be at h * 0.5; slab is 0.5 thick, centred half below it.
        assert_eq!(placement.translation.y + placement.scale.y * 0.5, 1.5);
    }

    #[test]
    fn column_fills_down_to_foundation() {
        let placement = column_placement(0, 0, &cell(3, TileType::Cube)).expect("column");
        let bottom = placement.translation.y - placement.scale.y * 0.5;
        assert!((bottom - FOUNDATION_BOTTOM).abs() < 1e-5);
    }

    #[test]
    fn flat_cell_has_no_column() {
        assert!(column_placement(0, 0, &cell(0, TileType::Cube)).is_none());
    }
}
