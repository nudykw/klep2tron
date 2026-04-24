use bevy::prelude::*;
use super::common::Tooltip;

#[derive(Component, Default)]
pub struct ColorPickerButton;

#[derive(Component, Default)]
pub struct ColorHueSlider;

#[derive(Component)]
pub struct ColorPreset(pub Color);

impl From<Color> for ColorPreset {
    fn from(color: Color) -> Self {
        Self(color)
    }
}

#[derive(Component, Default)]
pub struct ColorPickerContainer;

pub fn spawn_color_picker(parent: &mut ChildBuilder, _font: &Handle<Font>, initial_color: Color, is_open: bool) {
    spawn_color_picker_ext::<ColorPickerButton, ColorPickerContainer, ColorHueSlider, ColorPreset>(parent, initial_color, is_open);
}

pub fn spawn_color_picker_ext<B, C, H, P>(
    parent: &mut ChildBuilder, 
    initial_color: Color, 
    is_open: bool
) where 
    B: Component + Default, 
    C: Component + Default, 
    H: Component + Default, 
    P: Component + From<Color> 
{
    let display = if is_open { Display::Flex } else { Display::None };
    parent.spawn((
        ButtonBundle { 
            style: Style { 
                width: Val::Percent(100.0), 
                height: Val::Px(32.0), 
                margin: UiRect::bottom(Val::Px(10.0)), 
                ..default() 
            }, 
            background_color: initial_color.into(), 
            border_radius: BorderRadius::all(Val::Px(4.0)), 
            ..default() 
        }, 
        B::default(), 
        Tooltip("Click to toggle Color Picker".to_string()), 
    ));
    
    parent.spawn((
        NodeBundle { 
            style: Style { 
                width: Val::Percent(100.0), 
                flex_direction: FlexDirection::Column, 
                display, 
                ..default() 
            }, 
            ..default() 
        }, 
        C::default(), 
    )).with_children(|container| {
        container.spawn((
            NodeBundle { 
                style: Style { 
                    width: Val::Percent(100.0), 
                    height: Val::Px(12.0), 
                    margin: UiRect::vertical(Val::Px(5.0)), 
                    ..default() 
                }, 
                background_color: Color::WHITE.into(), 
                ..default() 
            }, 
            H::default(), 
            Interaction::default(), 
            Tooltip("Slide to change Hue".to_string()), 
        ));
        
        container.spawn(NodeBundle { 
            style: Style { 
                width: Val::Percent(100.0), 
                display: Display::Grid, 
                grid_template_columns: vec![GridTrack::flex(1.0); 5], 
                column_gap: Val::Px(4.0), 
                row_gap: Val::Px(4.0), 
                margin: UiRect::top(Val::Px(5.0)), 
                ..default() 
            }, 
            ..default() 
        }).with_children(|grid| {
            let presets = [
                Color::srgb(0.1, 0.1, 0.1), Color::srgb(0.5, 0.5, 0.5), Color::srgb(0.9, 0.9, 0.9), 
                Color::srgb(1.0, 0.8, 0.2), Color::srgb(0.8, 0.8, 0.9), Color::srgb(0.8, 0.2, 0.2), 
                Color::srgb(0.2, 0.8, 0.2), Color::srgb(0.2, 0.2, 0.8), Color::srgb(0.8, 0.4, 1.0), 
                Color::srgb(1.0, 0.5, 0.0), 
            ];
            for color in presets { 
                grid.spawn((
                    ButtonBundle { 
                        style: Style { width: Val::Px(24.0), height: Val::Px(24.0), ..default() }, 
                        background_color: color.into(), 
                        border_radius: BorderRadius::all(Val::Px(4.0)), 
                        ..default() 
                    }, 
                    P::from(color), 
                )); 
            }
        });
    });
}
