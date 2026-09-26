//! Spawning the player hero from its baked `.k2m` parts.

use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use shared::npc::ActorPart;

use crate::actor::ActorManifest;
use crate::actor::ActorRoot;
use crate::{ClientAssets, Project};

/// Marks that the hero has been spawned for the current game session.
///
/// Reset when leaving [`GameState::InGame`](crate::GameState).
#[derive(Resource, Default)]
pub struct PlayerActorSpawned(pub bool);

/// Cell `(X, Z)` where the hero appears.
const SPAWN_CELL: (usize, usize) = (2, 2);

/// Lowest local-space Y of a mesh (its "feet"), if positions are available.
fn mesh_min_y(mesh: &Mesh) -> Option<f32> {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(positions)) => {
            positions.iter().map(|position| position[1]).reduce(f32::min)
        }
        _ => None,
    }
}

/// Spawns the hero once its assets are ready.
///
/// Runs every frame while in game and is a no-op after the first success, so a
/// late-finishing asset load is handled without extra state machine.
pub fn spawn_player_actor(
    mut commands: Commands,
    mut spawned: ResMut<PlayerActorSpawned>,
    assets: Res<ClientAssets>,
    manifests: Res<Assets<ActorManifest>>,
    meshes: Res<Assets<Mesh>>,
    project: Res<Project>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if spawned.0 {
        return;
    }

    let Some(manifest) = manifests.get(&assets.actor_manifest) else {
        return;
    };
    if assets.actor_head == Handle::default() {
        return;
    }

    let (cell_x, cell_z) = SPAWN_CELL;
    let surface_y = project
        .rooms
        .get(project.current_room_idx)
        .map(|room| room.cells[cell_x][cell_z].h.max(0) as f32 * 0.5)
        .unwrap_or(0.0);

    // Rest the model on the floor: offset by its lowest vertex, scaled by the
    // manifest scale. Falls back to no offset when the meshes are unavailable.
    let scale = manifest.0.scale;
    let feet_y = [&assets.actor_head, &assets.actor_body, &assets.actor_legs]
        .into_iter()
        .filter_map(|handle| meshes.get(handle))
        .filter_map(mesh_min_y)
        .reduce(f32::min)
        .unwrap_or(0.0);

    let root = commands
        .spawn((
            ActorRoot,
            Name::new(format!("Actor: {}", manifest.0.name)),
            Transform::from_xyz(cell_x as f32, surface_y - feet_y * scale.y, cell_z as f32)
                .with_scale(scale),
            Visibility::default(),
        ))
        .id();

    let parts = [
        (ActorPart::Engine, assets.actor_legs.clone(), Color::srgb(1.0, 0.6, 0.2), "Engine"),
        (ActorPart::Body, assets.actor_body.clone(), Color::srgb(0.8, 0.8, 0.8), "Body"),
        (ActorPart::Head, assets.actor_head.clone(), Color::srgb(0.3, 0.6, 1.0), "Head"),
    ];

    for (part, mesh, color, label) in parts {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.5,
                ..default()
            })),
            part,
            Name::new(label),
            ChildOf(root),
        ));
    }

    spawned.0 = true;
    info!("Spawned hero '{}' at cell ({}, {})", manifest.0.name, cell_x, cell_z);
}

/// Despawns the hero hierarchy (Bevy despawns `Children` together with the root).
pub fn despawn_player_actor(
    mut commands: Commands,
    roots: Query<Entity, With<ActorRoot>>,
    mut spawned: ResMut<PlayerActorSpawned>,
) {
    for entity in roots.iter() {
        commands.entity(entity).despawn();
    }
    spawned.0 = false;
}
