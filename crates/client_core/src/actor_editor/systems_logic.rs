use bevy::prelude::*;
use crate::GameState;
use super::{ActorEditorBackButton, ViewportSettings, ResetCameraEvent, MainEditorCamera, GizmoCamera};

pub fn actor_editor_input_system(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ActorEditorBackButton>)>,
    mut viewport_settings: ResMut<ViewportSettings>,
    mut reset_events: EventWriter<ResetCameraEvent>,
) {
    // Menu navigation
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }

    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Menu);
        }
    }

    // Viewport Hotkeys
    if keyboard.just_pressed(KeyCode::KeyG) {
        viewport_settings.grid = !viewport_settings.grid;
    }
    if keyboard.just_pressed(KeyCode::KeyS) {
        viewport_settings.slices = !viewport_settings.slices;
    }
    if keyboard.just_pressed(KeyCode::KeyK) {
        viewport_settings.sockets = !viewport_settings.sockets;
    }
    if keyboard.just_pressed(KeyCode::KeyZ) {
        viewport_settings.gizmos = !viewport_settings.gizmos;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        reset_events.send(ResetCameraEvent);
    }
}

pub fn gizmo_sync_system(
    main_camera: Query<&Transform, (With<MainEditorCamera>, Without<GizmoCamera>)>,
    mut gizmo_camera: Query<&mut Transform, With<GizmoCamera>>,
) {
    if let Ok(main_transform) = main_camera.get_single() {
        if let Ok(mut gizmo_transform) = gizmo_camera.get_single_mut() {
            // Only sync rotation, keep gizmo camera at its fixed distance
            gizmo_transform.rotation = main_transform.rotation;
        }
    }
}

pub fn camera_reset_system(
    mut reset_events: EventReader<ResetCameraEvent>,
    mut camera_query: Query<&mut Transform, With<MainEditorCamera>>,
) {
    for _ in reset_events.read() {
        if let Ok(mut transform) = camera_query.get_single_mut() {
            *transform = Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
        }
    }
}

pub fn gizmo_viewport_system(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut gizmo_camera: Query<&mut Camera, With<GizmoCamera>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Ok(mut camera) = gizmo_camera.get_single_mut() else { return; };
    
    let size = 120;
    let padding = 20;
    
    camera.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(padding, window.physical_height().saturating_sub(size + padding)),
        physical_size: UVec2::new(size, size),
        depth: 0.0..1.0,
    });
}
