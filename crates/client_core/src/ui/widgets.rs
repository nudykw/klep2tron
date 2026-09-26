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
///
/// NOTE: `GlobalZIndex` is deliberately *not* a field here. A `GlobalZIndex`
/// component (even `GlobalZIndex(0)`) makes `bevy_ui`'s `ui_stack_system`
/// treat the node as a separate stacking root and detach it from its parent's
/// traversal, which makes draw order depend on archetype order and therefore
/// on unrelated archetype moves (e.g. `bevy_picking` inserting
/// `PickingInteraction` on first hover). Add `GlobalZIndex(n)` as a separate
/// component only when you actually need a global overlay.
#[derive(Bundle, Default)]
pub struct UiNode {
    pub node: Node,
    pub background_color: BackgroundColor,
    pub border_color: BorderColor,
    pub z_index: ZIndex,
    pub visibility: Visibility,
}
