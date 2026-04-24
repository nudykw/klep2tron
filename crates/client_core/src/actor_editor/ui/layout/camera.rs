use bevy::prelude::*;
use bevy::render::camera::ClearColorConfig;
use crate::actor_editor::{ActorEditorEntity, MainEditorCamera, GizmoCamera, GIZMO_LAYER};

pub fn spawn_actor_editor_cameras(commands: &mut Commands) -> Entity {
    // 3D Main Camera
    let main_camera_entity = commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 5,
                clear_color: Color::BLACK.into(),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
        MainEditorCamera,
    )).id();

    // Gizmo Camera (Sub-view)
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 10,
                viewport: Some(bevy::render::camera::Viewport {
                    physical_position: UVec2::new(20, 20),
                    physical_size: UVec2::new(120, 120),
                    depth: 0.0..1.0,
                }),
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera_3d: Camera3d::default(),
            transform: Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
        GizmoCamera,
        GIZMO_LAYER,
    ));

    main_camera_entity
}

pub fn spawn_actor_editor_lighting(commands: &mut Commands, main_camera_entity: Entity) {
    // --- 3-POINT LIGHTING ---
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 25000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 12000.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(-5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 15000.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 5.0, -8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    commands.entity(main_camera_entity).with_children(|parent| {
        parent.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 80000.0,
                range: 15.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(0.8, 0.8, 0.0),
            ..default()
        });
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
    });
}
