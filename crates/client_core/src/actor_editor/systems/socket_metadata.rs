use bevy::prelude::*;
use super::super::{ActorSocket, ToastEvent, ToastType};
use super::super::ui_inspector::{SelectedSocket, SocketNameInput, SocketCommentInput, SocketDetailsContainer};
use super::super::widgets::TextInput;

pub fn socket_metadata_sync_system(
    selected: Res<SelectedSocket>,
    mut socket_query: Query<(Entity, &mut ActorSocket)>,
    mut name_input_query: Query<&mut TextInput, (With<SocketNameInput>, Without<SocketCommentInput>)>,
    mut comment_input_query: Query<&mut TextInput, (With<SocketCommentInput>, Without<SocketNameInput>)>,
    mut container_query: Query<&mut Style, With<SocketDetailsContainer>>,
    mut last_selected: Local<Option<Entity>>,
    _toast_events: EventWriter<ToastEvent>,
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
            *last_selected = Some(selected_entity);
        }
        return;
    }

    // Handle updates from inputs to socket data
    let name_changed = name_input.is_changed();
    let comment_changed = comment_input.is_changed();

    if name_changed || comment_changed {
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

        // If user tries to confirm invalid name (e.g. loses focus or hits Enter)
        // We can check this in text_input_system or here if we detect focus loss
        // For now, let's just update if valid
        if let Ok((_, mut socket)) = socket_query.get_mut(selected_entity) {
            if is_unique {
                socket.definition.name = name_input.value.clone();
            }
            socket.definition.comment = comment_input.value.clone();
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
