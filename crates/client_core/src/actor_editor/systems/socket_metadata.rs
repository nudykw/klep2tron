use bevy::prelude::*;
use super::super::{ActorSocket, ToastEvent, ToastType, SocketColorPickerState, SocketColorPicker, SocketColorPickerContainer, SocketColorHueSlider, SocketColorPreset};
use super::super::ui::inspector::{SelectedSocket, SocketNameInput, SocketCommentInput, SocketDetailsContainer};
use super::super::widgets::TextInput;

pub fn socket_metadata_sync_system(
    selected: Res<SelectedSocket>,
    mut socket_query: Query<(Entity, &mut ActorSocket)>,
    mut name_input_query: Query<&mut TextInput, (With<SocketNameInput>, Without<SocketCommentInput>)>,
    mut comment_input_query: Query<&mut TextInput, (With<SocketCommentInput>, Without<SocketNameInput>)>,
    mut container_query: Query<&mut Style, With<SocketDetailsContainer>>,
    mut last_selected: Local<Option<Entity>>,
    _toast_events: EventWriter<ToastEvent>,
    mut color_state: ResMut<SocketColorPickerState>,
) {
    let Ok(mut container_style) = container_query.get_single_mut() else { return; };

    let Some(selected_entity) = selected.0 else {
        container_style.display = Display::None;
        *last_selected = None;
        return;
    };

    container_style.display = Display::Flex;

    let Ok(mut name_input) = name_input_query.get_single_mut() else { return; };
    let Ok(mut comment_input) = comment_input_query.get_single_mut() else { return; };

    // If selection changed, populate inputs from socket data
    if Some(selected_entity) != *last_selected {
        if let Ok((_, socket)) = socket_query.get(selected_entity) {
            name_input.value = socket.definition.name.clone();
            name_input.is_valid = true;
            comment_input.value = socket.definition.comment.clone();
            
            // Sync color picker state
            color_state.color = socket.definition.color;
            let hsla = Hsla::from(socket.definition.color);
            color_state.hue = hsla.hue;
            
            *last_selected = Some(selected_entity);
        }
        return;
    }

    // Handle updates from inputs to socket data
    let name_changed = name_input.is_changed();
    let comment_changed = comment_input.is_changed();
    let color_changed = color_state.is_changed();

    if name_changed || comment_changed || color_changed {
        // Validation: Name uniqueness
        let mut is_unique = true;
        let mut part = None;
        
        if let Ok((_, socket)) = socket_query.get(selected_entity) {
            part = Some(socket.definition.part);
        }

        if let Some(part) = part {
            for (entity, other_socket) in socket_query.iter() {
                if entity != selected_entity && other_socket.definition.part == part {
                    if other_socket.definition.name == name_input.value {
                        is_unique = false;
                        break;
                    }
                }
            }
        }

        if name_input.value.is_empty() {
            is_unique = false;
        }

        name_input.is_valid = is_unique;

        if let Ok((_, mut socket)) = socket_query.get_mut(selected_entity) {
            if is_unique {
                socket.definition.name = name_input.value.clone();
            }
            socket.definition.comment = comment_input.value.clone();
            socket.definition.color = color_state.color;
        }
    }
}

pub fn socket_validation_feedback_system(
    name_input_query: Query<&TextInput, (With<SocketNameInput>, Changed<TextInput>)>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for input in name_input_query.iter() {
        if !input.is_focused && !input.is_valid && !input.value.is_empty() {
             toast_events.send(ToastEvent {
                message: format!("Duplicate name: '{}'", input.value),
                toast_type: ToastType::Error,
            });
        }
    }
}

pub fn socket_color_picker_system(
    mut color_state: ResMut<SocketColorPickerState>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<SocketColorPicker>)>,
    hue_query: Query<(&Interaction, &Node, &GlobalTransform), With<SocketColorHueSlider>>,
    preset_query: Query<(&Interaction, &SocketColorPreset)>,
    mut container_query: Query<&mut Style, With<SocketColorPickerContainer>>,
    mut preview_query: Query<&mut BackgroundColor, (With<SocketColorPicker>, Without<SocketColorPreset>)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            color_state.is_open = !color_state.is_open;
            if let Ok(mut style) = container_query.get_single_mut() {
                style.display = if color_state.is_open { Display::Flex } else { Display::None };
            }
        }
    }

    let Ok(window) = window_query.get_single() else { return; };
    if let Some(cursor) = window.cursor_position() {
        for (interaction, node, transform) in hue_query.iter() {
            if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
                if *interaction == Interaction::Pressed {
                    let rect = node.size();
                    let pos = transform.translation().truncate();
                    let local_x = cursor.x - (pos.x - rect.x / 2.0);
                    let hue = (local_x / rect.x).clamp(0.0, 1.0) * 360.0;
                    color_state.hue = hue;
                    color_state.color = Color::hsla(hue, 0.8, 0.5, 1.0);
                }
            }
        }
    }

    for (interaction, preset) in preset_query.iter() {
        if *interaction == Interaction::Pressed {
            color_state.color = preset.0;
            let hsla = Hsla::from(preset.0);
            color_state.hue = hsla.hue;
        }
    }

    if color_state.is_changed() {
        if let Ok(mut bg) = preview_query.get_single_mut() {
            bg.0 = color_state.color;
        }
        
        // Also ensure container visibility matches state (useful when selection changes)
        if let Ok(mut style) = container_query.get_single_mut() {
            let target_display = if color_state.is_open { Display::Flex } else { Display::None };
            if style.display != target_display {
                style.display = target_display;
            }
        }
    }
}

pub fn socket_material_sync_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    socket_query: Query<(Entity, &ActorSocket), Changed<ActorSocket>>,
    children_query: Query<&Children>,
    material_handle_query: Query<&Handle<StandardMaterial>>,
) {
    for (socket_entity, socket) in socket_query.iter() {
        let color = socket.definition.color;
        
        // Update the socket's own material (Torus)
        if let Ok(mat_handle) = material_handle_query.get(socket_entity) {
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.base_color = color;
            }
        }
        
        // Update children materials (Pin/Cone)
        if let Ok(children) = children_query.get(socket_entity) {
            for &child in children.iter() {
                if let Ok(mat_handle) = material_handle_query.get(child) {
                    if let Some(mat) = materials.get_mut(mat_handle) {
                        mat.base_color = color;
                    }
                }
            }
        }
    }
}
