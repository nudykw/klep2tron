use bevy::prelude::*;
use bevy::camera::ClearColorConfig;
use crate::{ActorEditorEntity, MainEditorCamera, GizmoCamera, GIZMO_LAYER};

pub fn spawn_actor_editor_cameras(commands: &mut Commands) -> Entity {
    // 3D Main Camera
    let main_camera_entity = commands.spawn((
        (Camera3d::default(), Camera {
                order: 5,
                clear_color: Color::BLACK.into(),
                ..default()
            }, Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y)),
        ActorEditorEntity,
        MainEditorCamera,
    )).id();

    // Gizmo Camera (Sub-view)
    commands.spawn((
        (Camera3d::default(), Camera {
                order: 10,
                viewport: Some(bevy::camera::Viewport {
                    physical_position: UVec2::new(20, 20),
                    physical_size: UVec2::new(120, 120),
                    depth: 0.0..1.0,
                }),
                clear_color: ClearColorConfig::None,
                ..default()
            }, Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y)),
        ActorEditorEntity,
        GizmoCamera,
        GIZMO_LAYER,
    ));

    main_camera_entity
}

pub fn spawn_actor_editor_lighting(commands: &mut Commands, main_camera_entity: Entity) {
    // --- 3-POINT LIGHTING ---
    commands.spawn((
        (DirectionalLight {
                illuminance: 25000.0,
                shadow_maps_enabled: true,
                ..default()
            }, Transform::from_xyz(4.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y)),
        ActorEditorEntity,
    ));

    commands.spawn((
        (DirectionalLight {
                illuminance: 12000.0,
                shadow_maps_enabled: false,
                ..default()
            }, Transform::from_xyz(-5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y)),
        ActorEditorEntity,
    ));

    commands.spawn((
        (DirectionalLight {
                illuminance: 15000.0,
                shadow_maps_enabled: false,
                ..default()
            }, Transform::from_xyz(0.0, 5.0, -8.0).looking_at(Vec3::ZERO, Vec3::Y)),
        ActorEditorEntity,
    ));

    commands.entity(main_camera_entity).with_children(|parent| {
        parent.spawn((PointLight {
                intensity: 80000.0,
                range: 15.0,
                shadow_maps_enabled: false,
                ..default()
            }, Transform::from_xyz(0.8, 0.8, 0.0)));
    });

    commands.entity(main_camera_entity).insert(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
        affects_lightmapped_meshes: false,
    });
}
