use bevy::prelude::*;
use crate::GameState;
use super::{ActorEditorBackButton, ViewportSettings, ResetCameraEvent, MainEditorCamera, GizmoCamera};

pub fn actor_editor_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ActorEditorBackButton>)>,
    mut viewport_settings: ResMut<ViewportSettings>,
    mut reset_events: EventWriter<ResetCameraEvent>,
    mut modal_events: EventWriter<super::ConfirmationRequestEvent>,
) {
    // Menu navigation
    let mut trigger_back = false;
    if keyboard.just_pressed(KeyCode::Escape) {
        trigger_back = true;
    }

    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            trigger_back = true;
        }
    }

    if trigger_back {
        modal_events.send(super::ConfirmationRequestEvent {
            title: "Discard Changes?".to_string(),
            message: "Are you sure you want to return to menu? Any unsaved changes will be lost.".to_string(),
            action: super::EditorAction::BackToMenu,
        });
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

pub fn status_update_system(
    status: Res<super::EditorStatus>,
    mut query: Query<&mut Text, With<super::widgets::StatusText>>,
) {
    if !status.is_changed() { return; }
    if let Ok(mut text) = query.get_single_mut() {
        let (val, color) = match *status {
            super::EditorStatus::Ready => ("READY", Color::srgb(0.8, 0.8, 0.8)),
            super::EditorStatus::Saving => ("SAVING...", Color::srgb(1.0, 0.8, 0.2)),
            super::EditorStatus::Loading => ("LOADING...", Color::srgb(0.2, 0.8, 1.0)),
            super::EditorStatus::Processing => ("PROCESSING...", Color::srgb(0.8, 0.4, 1.0)),
        };
        text.sections[0].value = val.to_string();
        text.sections[0].style.color = color;
    }
}

pub fn polycount_update_system(
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Handle<Mesh>, With<super::ActorEditorEntity>>,
    mut text_query: Query<&mut Text, With<super::widgets::PolycountText>>,
) {
    let mut total_polys = 0;
    for handle in mesh_query.iter() {
        if let Some(mesh) = meshes.get(handle) {
            if let Some(indices) = mesh.indices() {
                total_polys += indices.len() / 3;
            } else {
                // If no indices, assume it's a triangle list and use vertex count
                if let Some(pos) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    total_polys += pos.len() / 3;
                }
            }
        }
    }
    
    if let Ok(mut text) = text_query.get_single_mut() {
        let new_val = format!("POLYS: {}", total_polys);
        if text.sections[0].value != new_val {
            text.sections[0].value = new_val;
        }
    }
}

pub fn toast_manager_system(
    mut commands: Commands,
    mut toast_events: EventReader<super::ToastEvent>,
    asset_server: Res<AssetServer>,
    container_query: Query<Entity, With<super::widgets::ToastContainer>>,
    mut timer_query: Query<(Entity, &mut super::widgets::ToastTimer, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");

    // Spawn new toasts
    if let Ok(container) = container_query.get_single() {
        for event in toast_events.read() {
            commands.entity(container).with_children(|p| {
                super::widgets::spawn_toast_item(p, &font, &icon_font, &event.message, event.toast_type);
            });
        }
    }

    // Update timers and despawn
    for (entity, mut timer, mut bg) in timer_query.iter_mut() {
        timer.0.tick(time.delta());
        
        // Simple fade out in last 0.5s
        let rem = timer.0.remaining_secs();
        if rem < 0.5 {
            let alpha = (rem / 0.5).clamp(0.0, 1.0);
            bg.0.set_alpha(alpha * 0.95);
        }

        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn modal_manager_system(
    mut commands: Commands,
    mut modal_events: EventReader<super::ConfirmationRequestEvent>,
    asset_server: Res<AssetServer>,
    cancel_query: Query<&Interaction, (Changed<Interaction>, With<super::widgets::CancelModalButton>)>,
    confirm_query: Query<(&Interaction, &super::widgets::ConfirmModalButton), (Changed<Interaction>, With<super::widgets::ConfirmModalButton>)>,
    overlay_query: Query<Entity, With<super::widgets::ModalOverlay>>,
    camera_query: Query<Entity, With<super::MainEditorCamera>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");
    let target_camera = camera_query.get_single().ok();

    // Spawn modal
    for event in modal_events.read() {
        super::widgets::spawn_confirmation_modal(&mut commands, &font, &icon_font, &event.title, &event.message, event.action, target_camera);
    }

    // Handle Cancel
    for interaction in cancel_query.iter() {
        if *interaction == Interaction::Pressed {
            for entity in overlay_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    // Handle Confirm
    for (interaction, confirm) in confirm_query.iter() {
        if *interaction == Interaction::Pressed {
            match confirm.0 {
                super::EditorAction::BackToMenu => {
                    next_state.set(GameState::Menu);
                }
            }
            // Close modal
            for entity in overlay_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn color_picker_system(
    mut color_res: ResMut<super::EditorMaterialColor>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<super::widgets::ColorPickerButton>)>,
    hue_query: Query<(&Interaction, &Node, &GlobalTransform), With<super::widgets::ColorHueSlider>>,
    preset_query: Query<(&Interaction, &super::widgets::ColorPreset)>,
    mut container_query: Query<&mut Style, With<super::widgets::ColorPickerContainer>>,
    mut preview_query: Query<&mut BackgroundColor, (With<super::widgets::ColorPickerButton>, Without<super::widgets::ColorPreset>)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    // Toggle
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            color_res.is_open = !color_res.is_open;
            if let Ok(mut style) = container_query.get_single_mut() {
                style.display = if color_res.is_open { Display::Flex } else { Display::None };
            }
        }
    }

    // Hue Slider
    let Ok(window) = window_query.get_single() else { return; };
    if let Some(cursor) = window.cursor_position() {
        for (interaction, node, transform) in hue_query.iter() {
            if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
                if *interaction == Interaction::Pressed {
                    let rect = node.size();
                    let pos = transform.translation().truncate();
                    let local_x = cursor.x - (pos.x - rect.x / 2.0);
                    let hue = (local_x / rect.x).clamp(0.0, 1.0) * 360.0;
                    color_res.hue = hue;
                    color_res.color = Color::hsla(hue, 0.8, 0.5, 1.0);
                }
            }
        }
    }

    // Presets
    for (interaction, preset) in preset_query.iter() {
        if *interaction == Interaction::Pressed {
            color_res.color = preset.0;
        }
    }

    // Update Preview
    if color_res.is_changed() {
        if let Ok(mut bg) = preview_query.get_single_mut() {
            bg.0 = color_res.color;
        }
    }
}

pub fn material_sync_system(
    color_res: Res<super::EditorMaterialColor>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_query: Query<&Handle<StandardMaterial>, With<super::ActorEditorEntity>>,
) {
    if !color_res.is_changed() { return; }
    for handle in mesh_query.iter() {
        if let Some(mat) = materials.get_mut(handle) {
            mat.base_color = color_res.color;
        }
    }
}
