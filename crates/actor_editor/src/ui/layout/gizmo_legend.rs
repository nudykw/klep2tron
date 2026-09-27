use bevy::prelude::*;
use crate::{ActorEditorEntity, GizmoEntity, GIZMO_LAYER, EditorHelper};

const LEGEND_ROTATION_X: Quat = Quat::from_array([0.0, 0.70710677, 0.0, 0.70710677]); // Y-90 for X-axis
const LEGEND_ROTATION_Y: Quat = Quat::from_array([-0.70710677, 0.0, 0.0, 0.70710677]); // X-90 for Y-axis
const LEGEND_ROTATION_Z: Quat = Quat::IDENTITY;

pub fn spawn_gizmo_legend(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Spawn Gizmo Axes
    let mesh_handle = meshes.add(Mesh::from(Cuboid::new(0.02, 0.02, 0.8)));
    
    // X - Red (Point Right)
    commands.spawn((
        (Mesh3d(mesh_handle.clone()), MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.2, 0.2), unlit: true, ..default() })), Transform::from_rotation(LEGEND_ROTATION_X)
                        .with_translation(Vec3::X * 0.4)),
        ActorEditorEntity,
        GizmoEntity,
        EditorHelper,
        GIZMO_LAYER,
    ));

    // Y - Green (Point Up)
    commands.spawn((
        (Mesh3d(mesh_handle.clone()), MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgb(0.2, 1.0, 0.2), unlit: true, ..default() })), Transform::from_rotation(LEGEND_ROTATION_Y)
                        .with_translation(Vec3::Y * 0.4)),
        ActorEditorEntity,
        GizmoEntity,
        EditorHelper,
        GIZMO_LAYER,
    ));
    // Z - Blue (Point Forward)
    commands.spawn((
        (Mesh3d(mesh_handle.clone()), MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgb(0.2, 0.2, 1.0), unlit: true, ..default() })), Transform::from_rotation(LEGEND_ROTATION_Z)
                        .with_translation(Vec3::Z * 0.4)),
        ActorEditorEntity,
        GizmoEntity,
        EditorHelper,
        GIZMO_LAYER,
    ));
}
