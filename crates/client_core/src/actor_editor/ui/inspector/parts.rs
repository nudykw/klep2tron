use bevy::prelude::*;
use crate::actor_editor::{
    widgets::spawn_collapsible_section_ext,
    ActorPart,
};
use super::types::*;

pub fn spawn_parts_section(
    p: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    spawn_collapsible_section_ext(
        p,
        font,
        icon_font,
        "PARTS",
        false,
        PartsSectionMarker,
        |content| {
            for (part, label) in [
                (ActorPart::Head, "Head"),
                (ActorPart::Body, "Body"),
                (ActorPart::Engine, "Legs"),
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
                            crate::actor_editor::widgets::Tooltip("Focus camera on part".to_string()),
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
                            crate::actor_editor::widgets::Tooltip("Isolate part (Solo mode)".to_string()),
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
                        crate::actor_editor::widgets::Tooltip(tooltip.to_string()),
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
                crate::actor_editor::widgets::Tooltip("Toggle Inspection Mode (Master Switch)".to_string()),
            )).with_children(|b| {
                b.spawn(TextBundle::from_section(
                    "\u{f011}", // power icon
                    TextStyle { font: icon_font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                ));
            });
        }
    );
}
