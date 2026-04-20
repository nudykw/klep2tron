use bevy::prelude::*;
use crate::{EditorMode, RttCamera, RttCameraTarget, OverlayCamera, CameraDebugText};

#[derive(Component)] 
pub struct OrbitCamera { 
    pub center: Vec3, 
    pub radius: f32, 
    pub angle: f32, 
    pub height: f32 
}

pub fn attach_editor_camera(
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut Transform), (With<Camera>, Without<OrbitCamera>, Without<RttCamera>, Without<OverlayCamera>)>,
    editor_mode: Res<EditorMode>,
) {
    if !editor_mode.is_active { return; }
    
    if let Ok((entity, mut transform)) = camera_query.get_single_mut() {
        let center = Vec3::new(7.5, 0.0, 7.5);
        let orbit = OrbitCamera {
            center,
            radius: 17.5,
            angle: 6.9,
            height: 8.0,
        };
        
        let x = orbit.center.x + orbit.radius * orbit.angle.cos();
        let z = orbit.center.z + orbit.radius * orbit.angle.sin();
        transform.translation = Vec3::new(x, orbit.height, z);
        transform.look_at(orbit.center, Vec3::Y);
        
        commands.entity(entity).insert(orbit);
    }
}

pub fn camera_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
    mut text_query: Query<&mut Text, With<CameraDebugText>>
) {
    let dt = time.delta_seconds();
    let speed = 10.0;
    let rot_speed = 2.0;
    for (mut transform, mut orbit) in query.iter_mut() {
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            if keyboard.pressed(KeyCode::ArrowUp)   { orbit.radius -= speed * dt; }
            if keyboard.pressed(KeyCode::ArrowDown) { orbit.radius += speed * dt; }
            if keyboard.pressed(KeyCode::ArrowLeft) { orbit.angle += rot_speed * dt; }
            if keyboard.pressed(KeyCode::ArrowRight) { orbit.angle -= rot_speed * dt; }
            if keyboard.pressed(KeyCode::KeyQ)       { orbit.height += speed * dt; }
            if keyboard.pressed(KeyCode::KeyA)       { orbit.height -= speed * dt; }
        }
        orbit.radius = orbit.radius.max(1.0);
        let x = orbit.center.x + orbit.radius * orbit.angle.cos();
        let z = orbit.center.z + orbit.radius * orbit.angle.sin();
        *transform = Transform::from_xyz(x, orbit.height, z).looking_at(orbit.center, Vec3::Y);
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = format!("CAM: X:{:.1} Y:{:.1} Z:{:.1} R:{:.1} A:{:.1}", x, orbit.height, z, orbit.radius, orbit.angle);
        }
    }
}

pub fn sync_overlay_camera_system(
    main_query: Query<&Transform, (With<OrbitCamera>, Without<OverlayCamera>)>,
    mut overlay_query: Query<&mut Transform, With<OverlayCamera>>,
) {
    if let Ok(main_trans) = main_query.get_single() {
        if let Ok(mut over_trans) = overlay_query.get_single_mut() {
            *over_trans = *main_trans;
        }
    }
}

pub fn sync_rtt_cameras_system(
    main_cam_query: Query<&OrbitCamera>,
    mut rtt_query: Query<(&mut Transform, &RttCameraTarget), With<RttCamera>>,
) {
    let Ok(orbit) = main_cam_query.get_single() else { return };
    
    for (mut transform, target) in rtt_query.iter_mut() {
        let radius = 2.0; 
        let x = target.0.x + radius * orbit.angle.cos();
        let z = target.0.z + radius * orbit.angle.sin();
        let y = target.0.y + (orbit.height / orbit.radius) * radius; 
        
        *transform = Transform::from_xyz(x, y, z).looking_at(target.0, Vec3::Y);
    }
}
