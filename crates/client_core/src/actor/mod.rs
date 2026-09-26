//! Player/NPC actors: baked `.k2m` parts loaded as assets and spawned in game.
//!
//! The hero is authored in the separate `actor_editor` crate, baked to
//! `assets/actors/<Name>/{head,body,legs}.k2m` + `actor.ron`, and reassembled
//! here. See `docs/Actor_Storage_Format.md` and `plans/Player_Actor_Spawning.md`.

mod loader;
mod spawn;

pub use loader::{ActorManifest, ActorManifestLoader, K2mMeshLoader};
pub use spawn::{despawn_player_actor, spawn_player_actor, PlayerActorSpawned};

use bevy::prelude::*;

use crate::GameState;

/// Marks the root entity of an actor hierarchy.
#[derive(Component)]
pub struct ActorRoot;

/// Where the hero's baked parts live, relative to the asset root.
#[derive(Resource, Debug, Clone)]
pub struct PlayerActorConfig {
    /// Actor folder under `assets/`, e.g. `actors/Robot`.
    pub dir: String,
}

impl Default for PlayerActorConfig {
    fn default() -> Self {
        Self {
            dir: "actors/Robot".to_string(),
        }
    }
}

impl PlayerActorConfig {
    /// Asset path of the head mesh.
    pub fn head_mesh(&self) -> String {
        format!("{}/head.k2m", self.dir)
    }
    /// Asset path of the body mesh.
    pub fn body_mesh(&self) -> String {
        format!("{}/body.k2m", self.dir)
    }
    /// Asset path of the engine (legs) mesh.
    pub fn legs_mesh(&self) -> String {
        format!("{}/legs.k2m", self.dir)
    }
    /// Asset path of the RON manifest.
    pub fn manifest(&self) -> String {
        format!("{}/actor.ron", self.dir)
    }
}

/// Registers actor asset loaders and the hero spawn systems.
pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ActorManifest>()
            .init_asset_loader::<K2mMeshLoader>()
            .init_asset_loader::<ActorManifestLoader>()
            .init_resource::<PlayerActorConfig>()
            .init_resource::<PlayerActorSpawned>()
            .add_systems(
                Update,
                spawn_player_actor.run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), despawn_player_actor);
    }
}
