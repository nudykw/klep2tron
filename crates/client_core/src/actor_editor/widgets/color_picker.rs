use bevy::prelude::*;
use super::common::Tooltip;

#[derive(Component)]
pub struct ColorPickerButton;

#[derive(Component)]
pub struct ColorHueSlider;

#[derive(Component)]
pub struct ColorPreset(pub Color);

#[derive(Component)]
pub struct ColorPickerContainer;

pub fn spawn_color_picker(parent: &mut ChildBuilder, _font: &Handle<Font>, initial_color: Color, is_open: bool) {
    let display = if is_open { Display::Flex } else { Display::None };
    parent.spawn((ButtonBundle { style: Style { width: Val::Percent(100.0), height: Val::Px(32.0), margin: UiRect::bottom(Val::Px(10.0)), ..default() }, background_color: initial_color.into(), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, ColorPickerButton, Tooltip("Click to toggle Color Picker".to_string()), ));
    parent.spawn((NodeBundle { style: Style { width: Val::Percent(100.0), flex_direction: FlexDirection::Column, display, ..default() }, ..default() }, ColorPickerContainer, )).with_children(|container| {
        container.spawn((NodeBundle { style: Style { width: Val::Percent(100.0), height: Val::Px(12.0), margin: UiRect::vertical(Val::Px(5.0)), ..default() }, background_color: Color::WHITE.into(), ..default() }, ColorHueSlider, Interaction::default(), Tooltip("Slide to change Hue".to_string()), ));
        container.spawn(NodeBundle { style: Style { width: Val::Percent(100.0), display: Display::Grid, grid_template_columns: vec![GridTrack::flex(1.0); 5], column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), margin: UiRect::top(Val::Px(5.0)), ..default() }, ..default() }).with_children(|grid| {
            let presets = [Color::srgb(0.1, 0.1, 0.1), Color::srgb(0.5, 0.5, 0.5), Color::srgb(0.9, 0.9, 0.9), Color::srgb(1.0, 0.8, 0.2), Color::srgb(0.8, 0.8, 0.9), Color::srgb(0.8, 0.2, 0.2), Color::srgb(0.2, 0.8, 0.2), Color::srgb(0.2, 0.2, 0.8), Color::srgb(0.8, 0.4, 1.0), Color::srgb(1.0, 0.5, 0.0), ];
            for color in presets { grid.spawn((ButtonBundle { style: Style { width: Val::Px(24.0), height: Val::Px(24.0), ..default() }, background_color: color.into(), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, ColorPreset(color), )); }
        });
    });
}
