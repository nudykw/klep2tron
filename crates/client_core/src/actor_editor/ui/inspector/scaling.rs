use bevy::prelude::*;
use crate::actor_editor::{
    widgets::spawn_collapsible_section_ext,
};
use super::types::*;

pub fn spawn_scaling_section(
    p: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    spawn_collapsible_section_ext(
        p,
        font,
        icon_font,
        "MODEL DIMENSIONS",
        false,
        ScalingSectionMarker,
        |content| {
            // Dimensions Inputs
            content.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            }).with_children(|list| {
                // Width (X)
                spawn_dimension_input(list, font, "Width (X), m:", ScalingInputX, "Width in meters");
                // Height (Y)
                spawn_dimension_input(list, font, "Height (Y), m:", ScalingInputY, "Height in meters");
                // Length (Z)
                spawn_dimension_input(list, font, "Length (Z), m:", ScalingInputZ, "Length in meters");
            });

            // Action Row
            content.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                ..default()
            }).with_children(|row| {
                // Link Proportions Toggle
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(40.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    ScalingLinkToggle,
                    crate::actor_editor::widgets::Tooltip("Uniform Scaling (All dimensions equal)".to_string()),
                )).with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "\u{f076}", // magnet icon
                        TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::WHITE },
                    ));
                });

                // Apply Button
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            flex_grow: 1.0,
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(0.2, 0.5, 0.2, 0.6).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    ScalingApplyButton,
                    crate::actor_editor::widgets::Tooltip("Apply new dimensions to the model".to_string()),
                )).with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "APPLY",
                        TextStyle { font: font.clone(), font_size: 11.0, color: Color::WHITE, ..default() },
                    ));
                });
            });
        },
        |_header| {}
    );
}

fn spawn_dimension_input(
    p: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    marker: impl Component,
    tooltip: &str,
) {
    p.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        },
        Interaction::default(),
        crate::actor_editor::widgets::Tooltip(tooltip.to_string()),
    )).with_children(|row| {
        row.spawn(TextBundle::from_section(
            label,
            TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.8, 0.8, 0.8) },
        ));
        
        row.spawn((
            crate::actor_editor::widgets::TextInputBundle {
                button: ButtonBundle {
                    style: Style {
                        width: Val::Px(80.0),
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
                    value: "1.00".to_string(),
                    placeholder: "1.00".to_string(),
                    ..default()
                },
            },
            marker,
        )).with_children(|p| {
            p.spawn((
                TextBundle::from_section(
                    "1.00",
                    TextStyle {
                        font: font.clone(),
                        font_size: 13.0,
                        color: Color::WHITE,
                    },
                ),
                crate::actor_editor::widgets::TextInputContent,
            ));
        });
    });
}
