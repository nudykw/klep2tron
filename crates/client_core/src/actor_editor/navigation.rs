use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use super::{MainEditorCamera, ViewportSettings, ResetCameraEvent};

pub fn setup_navigation(
    mut commands: Commands,
    camera_query: Query<Entity, With<MainEditorCamera>>,
) {
    if let Ok(entity) = camera_query.get_single() {
        commands.entity(entity).insert(PanOrbitCamera {
            focus: Vec3::new(0.0, 1.0, 0.0),
            radius: Some(4.0),
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
            enabled: true,
            ..default()
        });
    }
}

pub fn grid_system(
    mut gizmos: Gizmos,
    settings: Res<ViewportSettings>,
) {
    if !settings.grid { return; }

    let color = Color::srgba(1.0, 1.0, 1.0, 0.1);
    let half_size = 5.0;
    let step = 1.0;

    for i in -5..=5 {
        let x = i as f32 * step;
        gizmos.line(
            Vec3::new(x, 0.0, -half_size),
            Vec3::new(x, 0.0, half_size),
            color,
        );
        
        let z = i as f32 * step;
        gizmos.line(
            Vec3::new(-half_size, 0.0, z),
            Vec3::new(half_size, 0.0, z),
            color,
        );
    }
}

pub fn camera_reset_handler(
    mut reset_events: EventReader<ResetCameraEvent>,
    mut camera_query: Query<&mut PanOrbitCamera, With<MainEditorCamera>>,
) {
    for _ in reset_events.read() {
        if let Ok(mut pan_orbit) = camera_query.get_single_mut() {
            pan_orbit.target_focus = Vec3::new(0.0, 1.0, 0.0);
            pan_orbit.target_radius = 4.0;
            pan_orbit.target_yaw = 0.0;
            pan_orbit.target_pitch = 0.0;
        }
    }
}

pub fn camera_control_blocking_system(
    mut camera_query: Query<&mut PanOrbitCamera, With<MainEditorCamera>>,
    ui_query: Query<&Interaction, With<Node>>,
) {
    let mut any_hovered = false;
    for interaction in ui_query.iter() {
        if *interaction != Interaction::None {
            any_hovered = true;
            break;
        }
    }
    
    if let Ok(mut camera) = camera_query.get_single_mut() {
        if camera.enabled == any_hovered {
            camera.enabled = !any_hovered;
        }
    }
}
