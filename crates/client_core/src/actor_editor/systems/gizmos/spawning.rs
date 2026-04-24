use bevy::prelude::*;
use super::super::super::{ActorSocket, ui::inspector::SelectedSocket};
use super::{GizmoAxisType, GizmoAction, SocketGizmo, SocketLink, GizmoAxis, ManualGizmoInteraction};

pub fn update_socket_gizmos_system(
    mut commands: Commands,
    selected: Res<SelectedSocket>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gizmo_query: Query<Entity, With<SocketGizmo>>,
    socket_query: Query<Entity, With<ActorSocket>>,
) {
    let selected_entity = selected.0;
    
    // Log selection state for debugging
    if selected.is_changed() {
        info!("SelectedSocket changed to: {:?}", selected_entity);
    }

    let mut needs_spawn = false;
    
    // Check if we need to despawn current gizmo
    for entity in gizmo_query.iter() {
        // If nothing selected or selection changed, despawn
        if selected_entity.is_none() || selected.is_changed() {
            commands.entity(entity).despawn_recursive();
            needs_spawn = selected_entity.is_some();
        }
    }

    // If nothing spawned yet but something is selected, spawn
    if gizmo_query.iter().count() == 0 && selected_entity.is_some() {
        needs_spawn = true;
    }

    if needs_spawn {
        if let Some(target) = selected_entity {
            if let Ok(_socket) = socket_query.get(target) {
                info!("Spawning Gizmo for socket: {:?}", target);
                // Spawn a new gizmo at root (decoupled)
                commands.spawn((
                    SpatialBundle::default(),
                    SocketGizmo,
                    SocketLink(target),
                    bevy_mod_picking::prelude::Pickable::default(),
                    Name::new("SocketGizmo"),
                    crate::actor_editor::ActorEditorEntity,
                )).with_children(|gizmo_root| {
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::X, GizmoAction::Translate, Color::srgb(1.0, 0.2, 0.2), target);
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::Y, GizmoAction::Translate, Color::srgb(0.2, 1.0, 0.2), target);
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::Z, GizmoAction::Translate, Color::srgb(0.2, 0.2, 1.0), target);

                    // Spawning rotation rings
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::X, GizmoAction::Rotate, Color::srgb(1.0, 0.2, 0.2), target);
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::Y, GizmoAction::Rotate, Color::srgb(0.2, 1.0, 0.2), target);
                    spawn_axis(gizmo_root, &mut meshes, &mut materials, GizmoAxisType::Z, GizmoAction::Rotate, Color::srgb(0.2, 0.2, 1.0), target);

                });
            }
        }
    }
}

pub fn spawn_axis(
    parent: &mut ChildBuilder,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    axis: GizmoAxisType,
    action: GizmoAction,
    color: Color,
    target: Entity,
) {
    match action {
        GizmoAction::Translate => {
            let rotation = match axis {
                GizmoAxisType::X => Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                GizmoAxisType::Y => Quat::IDENTITY,
                GizmoAxisType::Z => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            };

            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(0.015, 1.0)),
                    material: materials.add(StandardMaterial {
                        base_color: color,
                        ..default()
                    }),
                    transform: Transform::from_rotation(rotation).with_translation(rotation * Vec3::Y * 0.5),
                    ..default()
                },
                GizmoAxis { axis, action },
                ManualGizmoInteraction::default(),
                SocketLink(target),
                bevy_mod_picking::prelude::PickableBundle::default(),
            )).with_children(|axis_p| {
                // Cone tip
                axis_p.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cone { radius: 0.05, height: 0.15 }),
                        material: materials.add(StandardMaterial { 
                            base_color: color,
                            ..default() 
                        }),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..default()
                    },
                    GizmoAxis { axis, action },
                    ManualGizmoInteraction::default(),
                    SocketLink(target),
                    bevy_mod_picking::prelude::PickableBundle::default(),
                ));
            });
        },
        GizmoAction::Rotate => {
            let rotation = match axis {
                GizmoAxisType::X => Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                GizmoAxisType::Y => Quat::IDENTITY,
                GizmoAxisType::Z => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            };

            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Torus { minor_radius: 0.01, major_radius: 0.75 }),
                    material: materials.add(StandardMaterial {
                        base_color: color,
                        unlit: true,
                        ..default()
                    }),
                    transform: Transform::from_rotation(rotation),
                    ..default()
                },
                GizmoAxis { axis, action },
                ManualGizmoInteraction::default(),
                SocketLink(target),
                bevy_mod_picking::prelude::PickableBundle::default(),
            ));
        }
    }
}
