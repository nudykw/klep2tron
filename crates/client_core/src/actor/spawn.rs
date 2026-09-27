//! Spawning the player hero from its baked `.k2m` parts, as a physics body.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use shared::npc::ActorPart;

use crate::actor::ActorManifest;
use crate::actor::ActorRoot;
use crate::physics::PlayerBody;
use crate::{ClientAssets, Project};

/// Marks that the hero has been spawned for the current game session.
///
/// Reset when leaving [`GameState::InGame`](crate::GameState).
#[derive(Resource, Default)]
pub struct PlayerActorSpawned(pub bool);

/// Cell `(X, Z)` where the hero appears.
const SPAWN_CELL: (usize, usize) = (2, 2);

/// Height above the surface the hero is dropped from, in world units.
const DROP_HEIGHT: f32 = 2.0;

/// Combined local-space AABB `(min, max)` of the part meshes.
///
/// Uses explicit per-component comparisons: `Vec3::min`/`max` returned wrong
/// results for these larger vertex buffers under the workspace's feature set.
fn model_bounds(meshes: &Assets<Mesh>, handles: [&Handle<Mesh>; 3]) -> Option<(Vec3, Vec3)> {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    let mut found = false;

    for handle in handles {
        let mesh = meshes.get(handle)?;
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        for position in positions {
            for axis in 0..3 {
                if position[axis] < min[axis] {
                    min[axis] = position[axis];
                }
                if position[axis] > max[axis] {
                    max[axis] = position[axis];
                }
            }
            found = true;
        }
    }

    found.then(|| (Vec3::from_array(min), Vec3::from_array(max)))
}

/// Spawns the hero once its assets *and meshes* are ready.
///
/// The root is the physics body (collider in world units, scale `ONE`); the
/// baked model is a child pivot scaled by the manifest, so Avian never has to
/// scale a collider. The body is dropped from [`DROP_HEIGHT`], so it falls onto
/// the tile colliders.
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
    let Some((model_min, model_max)) = model_bounds(
        &meshes,
        [&assets.actor_head, &assets.actor_body, &assets.actor_legs],
    ) else {
        return;
    };

    let (cell_x, cell_z) = SPAWN_CELL;
    let surface_y = project
        .rooms
        .get(project.current_room_idx)
        .map(|room| room.cells[cell_x][cell_z].h.max(0) as f32 * 0.5)
        .unwrap_or(0.0);

    let scale = manifest.0.scale;

    // Cylindrical/capsule body sized from the model, in world units (GDD: cylinder).
    // Capsule provides rounded hemispherical ends so the body glides over tile seams.
    let height = ((model_max.y - model_min.y) * scale.y).max(0.1);
    let radius = (0.5 * (model_max.x - model_min.x).min(model_max.z - model_min.z) * scale.x)
        .max(0.05);

    let collider = if height > 2.0 * radius {
        Collider::capsule(radius, height - 2.0 * radius)
    } else {
        Collider::sphere(radius)
    };

    let root = commands
        .spawn((
            ActorRoot,
            Name::new(format!("Actor: {}", manifest.0.name)),
            Transform::from_xyz(
                cell_x as f32,
                surface_y + DROP_HEIGHT - model_min.y * scale.y,
                cell_z as f32,
            ),
            Visibility::default(),
            RigidBody::Dynamic,
            collider,
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            PlayerBody { radius, height },
            LockedAxes::ROTATION_LOCKED,
        ))
        .id();

    // The visual pivot carries the model scale, keeping the collider unscaled.
    let pivot = commands
        .spawn((
            Name::new("Visual"),
            Transform::from_scale(scale),
            Visibility::default(),
            ChildOf(root),
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
            ChildOf(pivot),
        ));
    }

    spawned.0 = true;
    info!(
        "Spawned hero '{}' at cell ({}, {}) — r={radius:.3}, h={height:.3}, dropping {DROP_HEIGHT} units",
        manifest.0.name, cell_x, cell_z
    );
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
