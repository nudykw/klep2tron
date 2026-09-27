use bevy::prelude::*;
use client_core::ui::widgets::*;
use crate::widgets::{spawn_collapsible_section, spawn_slider};

pub fn spawn_materials_section(
    p: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    spawn_collapsible_section(
        p,
        font,
        icon_font,
        "MATERIALS",
        true,
        (),
        |content| {
            content.spawn(ui_text("Color", &font.clone(), 14.0, Color::WHITE));
            crate::widgets::spawn_color_picker(content, font, Color::srgb(0.7, 0.7, 0.7), false);

            content.spawn(ui_text("Metallic", &font.clone(), 14.0, Color::WHITE));
            spawn_slider(content, 0.5);

            content.spawn(ui_text("Roughness", &font.clone(), 14.0, Color::WHITE));
            spawn_slider(content, 0.8);
        }
    );
}
