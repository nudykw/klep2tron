//! Small helpers for common UI spawn patterns.
//!
//! Bevy 0.15 removed the UI bundles in favour of required components.
//! These helpers keep the call sites short and readable.

use bevy::prelude::*;

/// A UI text label. Replaces the old
/// `TextBundle::from_section(content, TextStyle { font, font_size, color })`.
pub fn ui_text(
    content: impl Into<String>,
    font: &Handle<Font>,
    font_size: f32,
    color: Color,
) -> (Text, TextFont, TextColor) {
    (
        Text::new(content),
        TextFont {
            font: font.clone().into(),
            font_size: font_size.into(),
            ..default()
        },
        TextColor(color),
    )
}

/// A UI node. Replaces the common
/// `NodeBundle { style, background_color, border_color, z_index, .. }`.
///
/// `border_color` is applied to all four sides. `border_radius` lives on
/// [`Node`] itself in Bevy 0.19, so put it inside the `node` field.
#[derive(Bundle, Default)]
pub struct UiNode {
    pub node: Node,
    pub background_color: BackgroundColor,
    pub border_color: BorderColor,
    pub z_index: ZIndex,
    pub global_z_index: GlobalZIndex,
    pub visibility: Visibility,
}
