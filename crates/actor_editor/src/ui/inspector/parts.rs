use bevy::prelude::*;
use client_core::ui::widgets::*;
use crate::{
    widgets::spawn_collapsible_section_ext,
    ActorPart,
};
use super::types::*;

pub fn spawn_parts_section(
    p: &mut ChildSpawnerCommands,
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
                content.spawn(UiNode { node: Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(5.0)),
                        ..default()
                    }, ..default() }).with_children(|row| {
                    row.spawn(ui_text(label, &font.clone(), 14.0, Color::WHITE));

                    row.spawn(UiNode { node: Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(5.0),
                            ..default()
                        }, ..default() }).with_children(|btns| {
                        // Focus Button
                        btns.spawn((
                            (Button, UiNode { node: Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                                }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }),
                            PartFocusButton(part),
                            crate::widgets::Tooltip("Focus camera on part".to_string()),
                        )).with_children(|b| {
                            b.spawn(ui_text("\u{f140}", &icon_font.clone(), 12.0, Color::srgb(0.8, 0.8, 0.8)));
                        });

                        // Solo Button
                        btns.spawn((
                            (Button, UiNode { node: Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                                }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }),
                            PartSoloButton(part),
                            crate::widgets::Tooltip("Isolate part (Solo mode)".to_string()),
                        )).with_children(|b| {
                            b.spawn(ui_text("\u{f06e}", &icon_font.clone(), 12.0, Color::srgb(0.8, 0.8, 0.8)));
                        });
                    });
                });
            }

            // Inspection Toggles (Ghost, Wireframe, Normals)
            content.spawn(UiNode { node: Node {
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                }, ..default() }).with_children(|row| {
                for (toggle, icon, label, tooltip) in [
                    (InspectionToggleType::Ghost, "\u{f070}", "Ghost", "Toggle Ghosting mode (alpha 0.1)"),
                    (InspectionToggleType::Wireframe, "\u{f1b2}", "Wire", "Toggle Wireframe view"),
                    (InspectionToggleType::Normals, "\u{f201}", "Norm", "Toggle Normals visualization"),
                ] {
                    row.spawn((
                        (Button, UiNode { node: Node {
                                width: Val::Px(60.0),
                                height: Val::Px(25.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                            }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)), ..default() }),
                        InspectionToggle(toggle),
                        crate::widgets::Tooltip(tooltip.to_string()),
                    )).with_children(|b| {
                        b.spawn(ui_text(icon, &icon_font.clone(), 10.0, Color::srgb(0.6, 0.6, 0.6)));
                        b.spawn(ui_text(label, &font.clone(), 8.0, Color::srgb(0.6, 0.6, 0.6)));
                    });
                }
            });
        },
        |header| {
            header.spawn((
                (Button, UiNode { node: Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                    }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)), ..default() }),
                InspectionMasterToggle,
                crate::widgets::Tooltip("Toggle Inspection Mode (Master Switch)".to_string()),
            )).with_children(|b| {
                b.spawn(ui_text("\u{f011}", &icon_font.clone(), 12.0, Color::srgb(0.6, 0.6, 0.6)));
            });
        }
    );
}
