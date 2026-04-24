use bevy::prelude::*;
use super::{widgets::{spawn_collapsible_section, spawn_collapsible_section_ext, spawn_slider, ScrollingList}, ActorPart};

#[derive(Component)]
pub struct InspectorPanel;

#[derive(Component)]
pub struct SocketSearchInput;

#[derive(Component)]
pub struct SocketPartFilterButton(pub Option<ActorPart>);

#[derive(Resource, Default)]
pub struct SocketFilterState {
    pub search_text: String,
    pub part_filter: Option<ActorPart>,
}

#[derive(Component)]
pub struct SocketListItem(pub Entity);

#[derive(Component)]
pub struct SocketListItemLabel;

#[derive(Component)]
pub struct SocketListContainer;

#[derive(Component)]
pub struct SocketAddModeButton;

#[derive(Component)]
pub struct MaterialColorPreview;
 
#[derive(Component)]
pub struct SocketNameInput;
 
#[derive(Component)]
pub struct SocketCommentInput;
 
#[derive(Component)]
pub struct SocketDetailsContainer;

#[derive(Resource, Default)]
pub struct SelectedSocket(pub Option<Entity>);

#[derive(Component)]
pub enum TransformAxis {
    X, Y, Z
}

#[derive(Component)]
pub enum RotationAxis {
    Roll, Pitch, Yaw
}

#[derive(Component)]
pub struct SocketResetRotationButton;

#[derive(Component)]
pub struct PartFocusButton(pub ActorPart);

#[derive(Component)]
pub struct PartSoloButton(pub ActorPart);

#[derive(Component)]
pub struct InspectionMasterToggle;

#[derive(Component)]
pub struct PartsSectionMarker;

#[derive(Component)]
pub struct InspectionToggle(pub InspectionToggleType);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectionToggleType {
    Ghost,
    Wireframe,
    Normals,
}

pub fn setup_inspector(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            focus_policy: bevy::ui::FocusPolicy::Block,
            ..default()
        },
        InspectorPanel,
        Interaction::default(),
    )).with_children(|p| {
        // --- MATERIALS SECTION ---
        spawn_collapsible_section(
            p,
            font,
            icon_font,
            "MATERIALS",
            true,
            (),
            |content| {
                content.spawn(TextBundle::from_section(
                    "Color",
                    TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                ));
                super::widgets::spawn_color_picker(content, font, Color::srgb(0.7, 0.7, 0.7), false);

                content.spawn(TextBundle::from_section(
                    "Metallic",
                    TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                ));
                spawn_slider(content, 0.5);

                content.spawn(TextBundle::from_section(
                    "Roughness",
                    TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                ));
                spawn_slider(content, 0.8);
            }
        );
        
        // --- SOCKETS SECTION ---
        spawn_collapsible_section(
            p,
            font,
            icon_font,
            "SOCKETS",
            false,
            (),
            |content| {
                content.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(5.0),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                }).with_children(|row| {
                    row.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        SocketAddModeButton,
                        super::super::widgets::Tooltip("Toggle Socket Placement Mode".to_string()),
                    )).with_children(|b| {
                        b.spawn(TextBundle::from_section(
                            "\u{f067}", // plus icon
                            TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::WHITE },
                        ));
                    });

                    row.spawn((
                        super::widgets::TextInputBundle {
                            button: ButtonBundle {
                                style: Style {
                                    flex_grow: 1.0,
                                    height: Val::Px(30.0),
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                z_index: ZIndex::Global(100),
                                ..default()
                            },
                            input: super::widgets::TextInput {
                                placeholder: "Search sockets...".to_string(),
                                ..default()
                            },
                        },
                        SocketSearchInput,
                        bevy_mod_picking::prelude::PickableBundle::default(),
                    )).with_children(|search| {
                        search.spawn((
                            TextBundle {
                                text: Text::from_section(
                                    "\u{f002} ", // fa-search
                                    TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.5, 0.5, 0.5) },
                                ),
                                focus_policy: bevy::ui::FocusPolicy::Pass,
                                ..default()
                            },
                        ));
                        search.spawn((
                            TextBundle {
                                text: Text::from_section(
                                    "Search sockets...",
                                    TextStyle { font: font.clone(), font_size: 14.0, color: Color::srgb(0.5, 0.5, 0.5) },
                                ),
                                focus_policy: bevy::ui::FocusPolicy::Pass,
                                ..default()
                            },
                            super::widgets::TextInputContent,
                        ));
                    });
                });

                // Filter Buttons Row
                content.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        margin: UiRect::vertical(Val::Px(5.0)),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|btns| {
                    for (part, label) in [
                        (None, "ALL"),
                        (Some(ActorPart::Head), "HEAD"),
                        (Some(ActorPart::Body), "BODY"),
                        (Some(ActorPart::Engine), "ENG"),
                    ] {
                        btns.spawn((
                            ButtonBundle {
                                style: Style {
                                    flex_grow: 1.0,
                                    height: Val::Px(20.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                ..default()
                            },
                            SocketPartFilterButton(part),
                        )).with_children(|b| {
                            b.spawn(TextBundle::from_section(
                                label,
                                TextStyle { font: font.clone(), font_size: 9.0, color: Color::srgb(0.7, 0.7, 0.7) },
                            ));
                        });
                    }
                });

                content.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            max_height: Val::Px(200.0),
                            ..default()
                        },
                        ..default()
                    },
                    SocketListContainer,
                    ScrollingList::default(),
                ));
                
                // --- POSITION DISPLAY ---
                content.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                }).with_children(|row| {
                    for (axis, label) in [(TransformAxis::X, "X"), (TransformAxis::Y, "Y"), (TransformAxis::Z, "Z")] {
                        row.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(75.0),
                                    height: Val::Px(25.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            axis,
                        )).with_children(|box_| {
                            box_.spawn(TextBundle::from_section(
                                format!("{}: {:.2}", label, 0.0),
                                TextStyle { font: font.clone(), font_size: 11.0, color: Color::WHITE },
                            ));
                        });
                    }
                });

                // --- ROTATION DISPLAY ---
                content.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        margin: UiRect::top(Val::Px(5.0)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                }).with_children(|row| {
                    for (axis, label) in [(RotationAxis::Roll, "R"), (RotationAxis::Pitch, "P"), (RotationAxis::Yaw, "Y")] {
                        row.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(75.0),
                                    height: Val::Px(25.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(0.1, 0.1, 0.1, 0.4).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            axis,
                        )).with_children(|box_| {
                            box_.spawn(TextBundle::from_section(
                                format!("{}: {:.1}°", label, 0.0),
                                TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.8, 0.8, 1.0) },
                            ));
                        });
                    }
                });

                // --- RESET BUTTON ---
                content.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(25.0),
                            margin: UiRect::top(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    SocketResetRotationButton,
                )).with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "Reset Rotation",
                        TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.7, 0.7, 0.7) },
                    ));
                });

                // --- SOCKET DETAILS (Name & Comment) ---
                content.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::top(Val::Px(15.0)),
                            row_gap: Val::Px(8.0),
                            display: Display::None, // Hidden by default
                            ..default()
                        },
                        ..default()
                    },
                    SocketDetailsContainer,
                )).with_children(|details| {
                    details.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(5.0)),
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                        ..default()
                    });

                    details.spawn(TextBundle::from_section(
                        "Socket Name",
                        TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                    ));
                    
                    // Actually, I'll just keep it manual as in previous edit, but removing the redundant line
                    // We need to insert the component on the newly spawned entity. 
                    // Since spawn_text_input returns the entity, we can use details.entity(id).
                    // Or we can modify spawn_text_input to allow adding components, but let's keep it simple.
                    // Wait, ChildBuilder doesn't have .entity(id) in 0.14? Let's check.
                    // Actually, I'll just change spawn_text_input to not return entity and instead take a closure or just do it manually.
                    
                    // Actually, I'll just do it manually here for now to be safe.
                    details.spawn((
                        super::widgets::TextInputBundle {
                            button: ButtonBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(28.0),
                                    padding: UiRect::horizontal(Val::Px(8.0)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            input: super::widgets::TextInput {
                                placeholder: "Socket name...".to_string(),
                                ..default()
                            },
                        },
                        SocketNameInput,
                    )).with_children(|p| {
                        p.spawn((
                            TextBundle::from_section(
                                "Socket name...",
                                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.5, 0.5, 0.5) },
                            ),
                            super::widgets::TextInputContent,
                        ));
                    });

                    details.spawn(TextBundle::from_section(
                        "Comment",
                        TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                    ));
                    
                    details.spawn((
                        super::widgets::TextInputBundle {
                            button: ButtonBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(28.0),
                                    padding: UiRect::horizontal(Val::Px(8.0)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            input: super::widgets::TextInput {
                                placeholder: "Add a comment...".to_string(),
                                ..default()
                            },
                        },
                        SocketCommentInput,
                    )).with_children(|p| {
                        p.spawn((
                            TextBundle::from_section(
                                "Add a comment...",
                                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.5, 0.5, 0.5) },
                            ),
                            super::widgets::TextInputContent,
                        ));
                    });
                });
            }
        );

        // --- PARTS SECTION ---
        spawn_collapsible_section_ext(
            p,
            font,
            icon_font,
            "PARTS",
            false,
            PartsSectionMarker,
            |content| {
                for (part, label) in [
                    (super::ActorPart::Head, "Head"),
                    (super::ActorPart::Body, "Body"),
                    (super::ActorPart::Engine, "Legs"),
                ] {
                    content.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(30.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: UiRect::horizontal(Val::Px(5.0)),
                            ..default()
                        },
                        ..default()
                    }).with_children(|row| {
                        row.spawn(TextBundle::from_section(
                            label,
                            TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                        ));

                        row.spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(5.0),
                                ..default()
                            },
                            ..default()
                        }).with_children(|btns| {
                            // Focus Button
                            btns.spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                PartFocusButton(part),
                                super::super::widgets::Tooltip("Focus camera on part".to_string()),
                            )).with_children(|b| {
                                b.spawn(TextBundle::from_section(
                                    "\u{f140}", // bullseye
                                    TextStyle { font: icon_font.clone(), font_size: 12.0, color: Color::srgb(0.8, 0.8, 0.8) },
                                ));
                            });

                            // Solo Button
                            btns.spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                PartSoloButton(part),
                                super::super::widgets::Tooltip("Isolate part (Solo mode)".to_string()),
                            )).with_children(|b| {
                                b.spawn(TextBundle::from_section(
                                    "\u{f06e}", // eye
                                    TextStyle { font: icon_font.clone(), font_size: 12.0, color: Color::srgb(0.8, 0.8, 0.8) },
                                ));
                            });
                        });
                    });
                }

                // Inspection Toggles (Ghost, Wireframe, Normals)
                content.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                }).with_children(|row| {
                    for (toggle, icon, label, tooltip) in [
                        (InspectionToggleType::Ghost, "\u{f070}", "Ghost", "Toggle Ghosting mode (alpha 0.1)"),
                        (InspectionToggleType::Wireframe, "\u{f1b2}", "Wire", "Toggle Wireframe view"),
                        (InspectionToggleType::Normals, "\u{f201}", "Norm", "Toggle Normals visualization"),
                    ] {
                        row.spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(60.0),
                                    height: Val::Px(25.0),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            InspectionToggle(toggle),
                            super::super::widgets::Tooltip(tooltip.to_string()),
                        )).with_children(|b| {
                            b.spawn(TextBundle::from_section(
                                icon,
                                TextStyle { font: icon_font.clone(), font_size: 10.0, color: Color::srgb(0.6, 0.6, 0.6) },
                            ));
                            b.spawn(TextBundle::from_section(
                                label,
                                TextStyle { font: font.clone(), font_size: 8.0, color: Color::srgb(0.6, 0.6, 0.6) },
                            ));
                        });
                    }
                });
            },
            |header| {
                header.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(20.0),
                            height: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    InspectionMasterToggle,
                    super::super::widgets::Tooltip("Toggle Inspection Mode (Master Switch)".to_string()),
                )).with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "\u{f011}", // power icon
                        TextStyle { font: icon_font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                    ));
                });
            }
        );
    });
}

pub fn socket_ui_list_sync_system(
    mut commands: Commands,
    fonts: Res<super::EditorFonts>,
    container_query: Query<Entity, With<SocketListContainer>>,
    socket_query: Query<(Entity, &super::ActorSocket)>,
    list_items_query: Query<(Entity, &SocketListItem)>,
) {
    let Ok(container) = container_query.get_single() else { return; };
    
    // Simple reconciliation: check if we have an item for each socket
    let existing_entities: std::collections::HashSet<Entity> = list_items_query.iter().map(|(_, item)| item.0).collect();
    let current_sockets: Vec<(Entity, &super::ActorSocket)> = socket_query.iter().collect();
    
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
    socket_query: Query<&super::ActorSocket, Changed<super::ActorSocket>>,
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
    mut socket_query: Query<&mut Transform, With<super::ActorSocket>>,
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
    socket_query: Query<&super::ActorSocket>,
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
    search_query: Query<&super::widgets::TextInput, With<SocketSearchInput>>,
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
