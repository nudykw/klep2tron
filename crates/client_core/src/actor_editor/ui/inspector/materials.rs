use bevy::prelude::*;
use crate::actor_editor::widgets::{spawn_collapsible_section, spawn_slider};

pub fn spawn_materials_section(
    p: &mut ChildBuilder,
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
            content.spawn(TextBundle::from_section(
                "Color",
                TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
            ));
            crate::actor_editor::widgets::spawn_color_picker(content, font, Color::srgb(0.7, 0.7, 0.7), false);

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
}
