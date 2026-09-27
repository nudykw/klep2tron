//! Game physics: Avian setup and static tile colliders.
//!
//! Tile colliders are derived from the same [`crate::rendering::tile_geometry`]
//! placement as the tile meshes, so collision matches what is drawn.

mod colliders;

pub use colliders::{clear_tile_colliders, rebuild_tile_colliders, TileCollider, TileColliderRoom};

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::GameState;

/// Cylinder dimensions of a player body, mirrored from its model at spawn.
#[derive(Component, Clone, Copy)]
pub struct PlayerBody {
    pub radius: f32,
    pub height: f32,
}

/// Registers Avian physics and keeps tile colliders in sync with the room.
pub struct GamePhysicsPlugin;

impl Plugin for GamePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            // Gravity stays at Avian's default `(0, -9.81, 0)`.
            .init_resource::<TileColliderRoom>()
            .add_systems(
                Update,
                rebuild_tile_colliders.run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), clear_tile_colliders);
    }
}
