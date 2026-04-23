use bevy::prelude::*;
use super::super::SlicingSettings;
use super::common::Tooltip;

#[derive(Component)]
pub struct Slider {
    pub value: f32,
}

#[derive(Component)]
pub struct SliderThumb;

pub fn spawn_slider(
    parent: &mut ChildBuilder,
    initial_value: f32,
) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(20.0),
                margin: UiRect::vertical(Val::Px(5.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },
        Slider { value: initial_value },
        Interaction::default(),
    )).with_children(|p| {
        p.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(4.0),
                ..default()
            },
            background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
            ..default()
        }).with_children(|track| {
            track.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(12.0),
                        height: Val::Px(12.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(initial_value * 100.0),
                        top: Val::Px(-4.0),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                SliderThumb,
            ));
        });
    });
}

pub fn slider_system(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut interaction_query: Query<(&Interaction, &Node, &GlobalTransform, &mut Slider, &Children)>,
    mut thumb_query: Query<&mut Style, With<SliderThumb>>,
    node_query: Query<&Children>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };

    for (interaction, node, transform, mut slider, children) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
            if *interaction == Interaction::Pressed {
                let rect = node.size();
                let pos = transform.translation().truncate();
                let local_x = cursor_position.x - (pos.x - rect.x / 2.0);
                slider.value = (local_x / rect.x).clamp(0.0, 1.0);
            }

            for child in children.iter() {
                if let Ok(track_children) = node_query.get(*child) {
                    for &thumb_entity in track_children.iter() {
                        if let Ok(mut style) = thumb_query.get_mut(thumb_entity) {
                            style.left = Val::Percent(slider.value * 100.0);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct RangeSlider {
    pub min_value: f32,
    pub max_value: f32,
    pub hovered_thumb: Option<RangeSliderThumb>,
    pub dragging: Option<RangeSliderThumb>,
    pub is_vertical: bool,
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum RangeSliderThumb {
    Min,
    Max,
}

pub fn spawn_range_slider(
    parent: &mut ChildBuilder,
    icon_font: &Handle<Font>,
    initial_min: f32,
    initial_max: f32,
) {
    spawn_range_slider_internal(parent, icon_font, initial_min, initial_max, false);
}

pub fn spawn_vertical_range_slider(parent: &mut ChildBuilder, icon_font: &Handle<Font>, initial_min: f32, initial_max: f32) {
    spawn_range_slider_internal(parent, icon_font, initial_min, initial_max, true);
}

fn spawn_range_slider_internal(
    parent: &mut ChildBuilder, 
    icon_font: &Handle<Font>,
    initial_min: f32, 
    initial_max: f32, 
    vertical: bool
) {
    let container_style = if vertical {
        Style {
            width: Val::Px(30.0),
            flex_grow: 1.0,
            margin: UiRect::horizontal(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(10.0)),
            ..default()
        }
    } else {
        Style {
            width: Val::Percent(100.0),
            height: Val::Px(24.0),
            margin: UiRect::vertical(Val::Px(10.0)),
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(10.0)),
            ..default()
        }
    };

    parent.spawn((
        NodeBundle { 
            style: container_style, 
            background_color: Color::NONE.into(), 
            ..default() 
        },
        RangeSlider { 
            min_value: initial_min, 
            max_value: initial_max, 
            hovered_thumb: None, 
            dragging: None,
            is_vertical: vertical 
        },
        Interaction::default(),
    )).with_children(|p| {
        let track_style = if vertical {
            Style { width: Val::Px(4.0), height: Val::Percent(100.0), ..default() }
        } else {
            Style { width: Val::Percent(100.0), height: Val::Px(4.0), ..default() }
        };

        p.spawn(NodeBundle {
            style: track_style,
            background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
            ..default()
        }).with_children(|p| {
            let thumb_size = 24.0;
            let thumb_style = Style { 
                width: Val::Px(thumb_size), 
                height: Val::Px(thumb_size), 
                position_type: PositionType::Absolute, 
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default() 
            };

            // Min Thumb (Bottom Cut / Engine) - Points UP
            let mut min_style = thumb_style.clone();
            if vertical { 
                min_style.bottom = Val::Percent(initial_min * 100.0); 
                min_style.left = Val::Px(-12.0); 
                min_style.margin.bottom = Val::Px(-thumb_size / 2.0);
            } else { 
                min_style.left = Val::Percent(initial_min * 100.0); 
                min_style.top = Val::Px(-12.0); 
                min_style.margin.left = Val::Px(-thumb_size / 2.0);
            }

            p.spawn((
                NodeBundle { 
                    style: min_style, 
                    background_color: Color::NONE.into(), 
                    z_index: ZIndex::Local(2), 
                    ..default() 
                },
                Interaction::default(), 
                RangeSliderThumb::Min, 
                Tooltip("Bottom Cut (Engine)".to_string()),
            )).with_children(|btn| {
                btn.spawn(TextBundle::from_section(if vertical { "\u{f0d8}" } else { "\u{f0da}" }, TextStyle { 
                    font: icon_font.clone(), 
                    font_size: thumb_size, 
                    color: Color::srgb(1.0, 0.6, 0.2) 
                }));
            });

            // Max Thumb (Top Cut / Head) - Points DOWN
            let mut max_style = thumb_style;
            if vertical { 
                max_style.bottom = Val::Percent(initial_max * 100.0); 
                max_style.left = Val::Px(-12.0); 
                max_style.margin.bottom = Val::Px(-thumb_size / 2.0);
            } else { 
                max_style.left = Val::Percent(initial_max * 100.0); 
                max_style.top = Val::Px(-12.0); 
                max_style.margin.left = Val::Px(-thumb_size / 2.0);
            }

            p.spawn((
                NodeBundle { 
                    style: max_style, 
                    background_color: Color::NONE.into(), 
                    z_index: ZIndex::Local(2), 
                    ..default() 
                },
                Interaction::default(), 
                RangeSliderThumb::Max, 
                Tooltip("Top Cut (Head)".to_string()),
            )).with_children(|btn| {
                btn.spawn(TextBundle::from_section(if vertical { "\u{f0d7}" } else { "\u{f0d9}" }, TextStyle { 
                    font: icon_font.clone(), 
                    font_size: thumb_size, 
                    color: Color::srgb(0.3, 0.6, 1.0) 
                }));
            });
        });
    });
}

pub fn range_slider_system(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut interaction_query: Query<(&Interaction, &Node, &GlobalTransform, &mut RangeSlider, &Children)>,
    mut thumb_query: Query<(&Interaction, &mut Style, &RangeSliderThumb, &mut Tooltip)>,
    node_query: Query<&Children>,
    mut slicing_settings: ResMut<SlicingSettings>,
) {

    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    
    if slicing_settings.locked { 
        for (_interaction, _style, thumb, mut tooltip) in thumb_query.iter_mut() {
            let base = match thumb { RangeSliderThumb::Min => "Bottom Cut (Engine)", RangeSliderThumb::Max => "Top Cut (Head)" };
            tooltip.0 = format!("{} [LOCKED]", base);
        }
        return; 
    }

    for (interaction, node, transform, mut slider, children) in interaction_query.iter_mut() {
        let mut hovered = None;
        for &track_entity in children.iter() {
            if let Ok(track_children) = node_query.get(track_entity) {
                for &child in track_children.iter() {
                    if let Ok((thumb_interaction, _style, thumb_type, _tooltip)) = thumb_query.get(child) {
                        if *thumb_interaction != Interaction::None { hovered = Some(*thumb_type); }
                    }
                }
            }
        }
        slider.hovered_thumb = hovered;

        for &track_entity in children.iter() {
            if let Ok(track_children) = node_query.get(track_entity) {
                for &child in track_children.iter() {
                    if let Ok((_interaction, mut style, thumb, mut tooltip)) = thumb_query.get_mut(child) {
                        let pct = match thumb { RangeSliderThumb::Min => slider.min_value, RangeSliderThumb::Max => slider.max_value, } * 100.0;
                        if slider.is_vertical { 
                            style.bottom = Val::Percent(pct); 
                        } else { 
                            style.left = Val::Percent(pct); 
                        }
                        let base = match thumb { RangeSliderThumb::Min => "Bottom Cut (Engine)", RangeSliderThumb::Max => "Top Cut (Head)" };
                        tooltip.0 = format!("{}: {:.0}%", base, pct);
                    }
                }
            }
        }

        if *interaction == Interaction::Pressed {
            let rect = node.size();
            let pos = transform.translation().truncate();
            let val = if slider.is_vertical {
                let local_y = (pos.y + rect.y / 2.0) - cursor_position.y;
                (local_y / rect.y).clamp(0.0, 1.0)
            } else {
                let local_x = (cursor_position.x - (pos.x - rect.x / 2.0)) / rect.x;
                local_x.clamp(0.0, 1.0)
            };

            if slider.dragging.is_none() {
                let dist_min = (val - slider.min_value).abs();
                let dist_max = (val - slider.max_value).abs();
                if dist_min < dist_max {
                    slider.dragging = Some(RangeSliderThumb::Min);
                } else {
                    slider.dragging = Some(RangeSliderThumb::Max);
                }
            }

            if let Some(target) = slider.dragging {
                match target {
                    RangeSliderThumb::Min => slider.min_value = val.min(slider.max_value - 0.02),
                    RangeSliderThumb::Max => slider.max_value = val.max(slider.min_value + 0.02),
                }
                // Sync dragging state to settings for 3D helpers
                slicing_settings.dragging_gizmo = Some(match target {
                    RangeSliderThumb::Min => super::super::SlicingGizmoType::Bottom,
                    RangeSliderThumb::Max => super::super::SlicingGizmoType::Top,
                });
            }
        } else {
            slider.dragging = None;
            // Clear dragging state if THIS slider was dragging
            if slicing_settings.dragging_gizmo.is_some() {
                // We don't clear it here yet, because we need it for the release logic in slicing.rs
                // Actually, slicing.rs will handle the release logic.
            }
        }
    }
}

