use bevy::prelude::*;
use crate::actor_editor::{
    widgets::spawn_collapsible_section,
    SocketColorPicker, SocketColorPickerContainer, SocketColorHueSlider, SocketColorPreset
};
use super::types::*;

pub fn spawn_sockets_section(
    p: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    spawn_collapsible_section(
        p,
        font,
        icon_font,
        "SOCKETS",
        false,
        SocketsSectionMarker,
        |content| {
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
                
                details.spawn((
                    crate::actor_editor::widgets::TextInputBundle {
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
                        input: crate::actor_editor::widgets::TextInput {
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
                        crate::actor_editor::widgets::TextInputContent,
                    ));
                });

                details.spawn(TextBundle::from_section(
                    "Comment",
                    TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                ));

                details.spawn(TextBundle::from_section(
                    "Visual Color",
                    TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                ));
                
                crate::actor_editor::widgets::spawn_color_picker_ext::<
                    SocketColorPicker, 
                    SocketColorPickerContainer, 
                    SocketColorHueSlider, 
                    SocketColorPreset
                >(details, Color::srgb(0.2, 0.8, 0.2), false);
                
                details.spawn((
                    crate::actor_editor::widgets::TextInputBundle {
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
                        input: crate::actor_editor::widgets::TextInput {
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
                        crate::actor_editor::widgets::TextInputContent,
                    ));
                });
            });
        }
    );
}
