use bevy::prelude::*;
use super::types::*;

pub fn socket_ui_list_sync_system(
    mut commands: Commands,
    fonts: Res<crate::actor_editor::EditorFonts>,
    container_query: Query<Entity, With<SocketListContainer>>,
    socket_query: Query<(Entity, &crate::actor_editor::ActorSocket)>,
    list_items_query: Query<(Entity, &SocketListItem)>,
) {
    let Ok(container) = container_query.get_single() else { return; };
    
    // Simple reconciliation: check if we have an item for each socket
    let existing_entities: std::collections::HashSet<Entity> = list_items_query.iter().map(|(_, item)| item.0).collect();
    let current_sockets: Vec<(Entity, &crate::actor_editor::ActorSocket)> = socket_query.iter().collect();
    
    // If mismatch, rebuild the list (simplified approach for now)
    let current_entities: std::collections::HashSet<Entity> = current_sockets.iter().map(|(e, _)| *e).collect();
    
    if existing_entities != current_entities {
        // Despawn all existing items
        for (item_entity, _) in list_items_query.iter() {
            commands.entity(item_entity).despawn_recursive();
        }
        
        // Spawn new items
        commands.entity(container).with_children(|parent| {
            for (socket_entity, socket) in current_sockets {
                parent.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(25.0),
                            margin: UiRect::bottom(Val::Px(2.0)),
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.02).into(),
                        ..default()
                    },
                    SocketListItem(socket_entity),
                )).with_children(|item| {
                    item.spawn((
                        TextBundle::from_section(
                            &socket.definition.name,
                            TextStyle { font: fonts.regular.clone(), font_size: 14.0, color: Color::srgb(0.9, 0.9, 0.9) },
                        ),
                        SocketListItemLabel,
                    ));
                });
            }
        });
    }
}

pub fn socket_ui_list_label_sync_system(
    socket_query: Query<&crate::actor_editor::ActorSocket, Changed<crate::actor_editor::ActorSocket>>,
    list_items_query: Query<(&SocketListItem, &Children)>,
    mut label_query: Query<&mut Text, With<SocketListItemLabel>>,
) {
    for (item, children) in list_items_query.iter() {
        if let Ok(socket) = socket_query.get(item.0) {
            for &child in children.iter() {
                if let Ok(mut text) = label_query.get_mut(child) {
                    if text.sections[0].value != socket.definition.name {
                        text.sections[0].value = socket.definition.name.clone();
                    }
                }
            }
        }
    }
}

pub fn socket_list_click_system(
    mut selected: ResMut<SelectedSocket>,
    query: Query<(&Interaction, &SocketListItem), Changed<Interaction>>,
) {
    for (interaction, item) in query.iter() {
        if *interaction == Interaction::Pressed {
            selected.0 = Some(item.0);
        }
    }
}

pub fn socket_list_highlight_system(
    selected: Res<SelectedSocket>,
    mut query: Query<(&SocketListItem, &mut BackgroundColor, &Interaction)>,
) {
    for (item, mut bg, interaction) in query.iter_mut() {
        if selected.0 == Some(item.0) {
            *bg = Color::srgba(0.0, 0.6, 1.0, 0.3).into();
        } else if *interaction == Interaction::Hovered {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.1).into();
        } else {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.02).into();
        }
    }
}


pub fn socket_transform_update_system(
    selected: Res<SelectedSocket>,
    socket_query: Query<&Transform>,
    mut pos_axis_query: Query<(&TransformAxis, &Children)>,
    mut rot_axis_query: Query<(&RotationAxis, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    let Some(entity) = selected.0 else { return; };
    let Ok(transform) = socket_query.get(entity) else { return; };
    
    // Update Translation
    for (axis, children) in pos_axis_query.iter_mut() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(*child) {
                let (val, label) = match axis {
                    TransformAxis::X => (transform.translation.x, "X"),
                    TransformAxis::Y => (transform.translation.y, "Y"),
                    TransformAxis::Z => (transform.translation.z, "Z"),
                };
                text.sections[0].value = format!("{}: {:.2}", label, val);
            }
        }
    }

    // Update Rotation
    let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
    for (axis, children) in rot_axis_query.iter_mut() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(*child) {
                let (val, label) = match axis {
                    RotationAxis::Roll => (roll.to_degrees(), "R"),
                    RotationAxis::Pitch => (pitch.to_degrees(), "P"),
                    RotationAxis::Yaw => (yaw.to_degrees(), "Y"),
                };
                text.sections[0].value = format!("{}: {:.1}°", label, val);
            }
        }
    }
}

pub fn socket_reset_rotation_system(
    selected: Res<SelectedSocket>,
    mut socket_query: Query<&mut Transform, With<crate::actor_editor::ActorSocket>>,
    query: Query<&Interaction, (With<SocketResetRotationButton>, Changed<Interaction>)>,
) {
    let Some(entity) = selected.0 else { return; };
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut transform) = socket_query.get_mut(entity) {
                transform.rotation = Quat::IDENTITY;
            }
        }
    }
}

pub fn socket_filter_update_system(
    filter: Res<SocketFilterState>,
    socket_query: Query<&crate::actor_editor::ActorSocket>,
    mut list_items_query: Query<(&SocketListItem, &mut Visibility)>,
) {
    if !filter.is_changed() { return; }

    for (item, mut visibility) in list_items_query.iter_mut() {
        if let Ok(socket) = socket_query.get(item.0) {
            let matches_search = filter.search_text.is_empty() || 
                socket.definition.name.to_lowercase().contains(&filter.search_text.to_lowercase());
            
            let matches_part = filter.part_filter.is_none() || 
                Some(socket.definition.part) == filter.part_filter;

            if matches_search && matches_part {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn socket_filter_ui_system(
    mut filter: ResMut<SocketFilterState>,
    search_query: Query<&crate::actor_editor::widgets::TextInput, With<SocketSearchInput>>,
    mut btns_query: Query<(&SocketPartFilterButton, &Interaction, &mut BackgroundColor)>,
) {
    // 1. Update search text
    if let Ok(search) = search_query.get_single() {
        if search.value != filter.search_text {
            filter.search_text = search.value.clone();
        }
    }

    // 2. Update part filter and visuals
    for (btn, interaction, mut bg) in btns_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            filter.part_filter = btn.0;
        }

        if filter.part_filter == btn.0 {
            *bg = Color::srgba(0.1, 0.4, 0.6, 0.4).into();
        } else if *interaction == Interaction::Hovered {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.1).into();
        } else {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
        }
    }
}
