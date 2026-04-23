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

#[derive(Component)]
pub struct ConfirmationCircleUI(pub RangeSliderThumb);


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
        bevy::ui::RelativeCursorPosition::default(),
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
                bevy::ui::RelativeCursorPosition::default(),
            )).with_children(|btn| {

                // Confirmation Circle UI
                btn.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(-8.0),
                            top: Val::Px(-8.0),
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.2).into(),
                        border_radius: BorderRadius::all(Val::Px(20.0)),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    ConfirmationCircleUI(RangeSliderThumb::Min),
                    Interaction::default(),
                    bevy::ui::RelativeCursorPosition::default(),
                ));



                btn.spawn(TextBundle {
                    text: Text::from_section(if vertical { "\u{f0d8}" } else { "\u{f0da}" }, TextStyle { 
                        font: icon_font.clone(), 
                        font_size: thumb_size, 
                        color: Color::srgb(1.0, 0.6, 0.2) 
                    }),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                });


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
                bevy::ui::RelativeCursorPosition::default(),
            )).with_children(|btn| {

                // Confirmation Circle UI
                btn.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(-8.0),
                            top: Val::Px(-8.0),
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.2).into(),
                        border_radius: BorderRadius::all(Val::Px(20.0)),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    ConfirmationCircleUI(RangeSliderThumb::Max),
                    Interaction::default(),
                    bevy::ui::RelativeCursorPosition::default(),
                ));



                btn.spawn(TextBundle {
                    text: Text::from_section(if vertical { "\u{f0d7}" } else { "\u{f0d9}" }, TextStyle { 
                        font: icon_font.clone(), 
                        font_size: thumb_size, 
                        color: Color::srgb(0.3, 0.6, 1.0) 
                    }),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                });


            });

        });
    });
}

pub fn range_slider_system(
    mut interaction_query: Query<(&Interaction, &Node, &GlobalTransform, &mut RangeSlider, &Children, &bevy::ui::RelativeCursorPosition)>,
    mut thumb_query: Query<(&Interaction, &mut Style, &RangeSliderThumb, &mut Tooltip, &Children, &GlobalTransform, &bevy::ui::RelativeCursorPosition)>,
    mut circle_query: Query<(&ConfirmationCircleUI, &mut Visibility, &mut BackgroundColor, &Interaction, &bevy::ui::RelativeCursorPosition)>,

    node_query: Query<&Children>,
    mut slicing_settings: ResMut<SlicingSettings>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    
    if slicing_settings.locked { 
        for (_interaction, _style, thumb, mut tooltip, _children, _transform, _rp) in thumb_query.iter_mut() {
            let base = match thumb { RangeSliderThumb::Min => "Bottom Cut (Engine)", RangeSliderThumb::Max => "Top Cut (Head)" };
            tooltip.0 = format!("{} [LOCKED]", base);
        }
        return; 
    }


    for (interaction, _node, _transform, mut slider, children, rel_pos) in interaction_query.iter_mut() {
        let mut hovered = None;
        for &track_entity in children.iter() {
            if let Ok(track_children) = node_query.get(track_entity) {
                for &child in track_children.iter() {
                    if let Ok((thumb_interaction, _style, thumb_type, _tooltip, _children, _transform, _rp)) = thumb_query.get(child) {

                        if *thumb_interaction != Interaction::None { hovered = Some(*thumb_type); }
                    }

                }
            }
        }
        slider.hovered_thumb = hovered;

        for &track_entity in children.iter() {
            if let Ok(track_children) = node_query.get(track_entity) {
                for &child in track_children.iter() {
                    if let Ok((thumb_interaction, mut style, thumb, mut tooltip, thumb_children, _thumb_transform, _thumb_rel_pos)) = thumb_query.get_mut(child) {




                        let pct = match thumb { RangeSliderThumb::Min => slider.min_value, RangeSliderThumb::Max => slider.max_value, } * 100.0;
                        if slider.is_vertical { 
                            style.bottom = Val::Percent(pct); 
                        } else { 
                            style.left = Val::Percent(pct); 
                        }
                        let base = match thumb { RangeSliderThumb::Min => "Bottom Cut (Engine)", RangeSliderThumb::Max => "Top Cut (Head)" };
                        tooltip.0 = format!("{}: {:.0}%", base, pct);

                        // --- SYNC CIRCLES ---
                        for &circle_entity in thumb_children.iter() {
                            if let Ok((circle, mut vis, mut bg, circle_interaction, circle_rel_pos)) = circle_query.get_mut(circle_entity) {
                                let slicing_type = match circle.0 {
                                    RangeSliderThumb::Min => super::super::SlicingGizmoType::Bottom,
                                    RangeSliderThumb::Max => super::super::SlicingGizmoType::Top,
                                };
                                
                                let is_dragging = slicing_settings.dragging_gizmo == Some(slicing_type);
                                let is_pending = slicing_settings.needs_confirm && slicing_settings.dragging_gizmo == Some(slicing_type);
                                
                                if is_dragging || is_pending {
                                    *vis = Visibility::Visible;
                                    if circle_rel_pos.mouse_over() {
                                        *bg = Color::srgba(1.0, 1.0, 1.0, 0.2).into();
                                    } else {
                                        *bg = Color::srgba(0.5, 0.5, 0.5, 0.4).into();
                                    }
                                } else {
                                    *vis = Visibility::Hidden;
                                }

                                // Only trigger slice if the thumb ITSELF wasn't clicked
                                if is_pending && *circle_interaction == Interaction::Pressed && *thumb_interaction != Interaction::Pressed {

                                    slicing_settings.trigger_slice = true;
                                    info!("UI Circle Clicked (Outside Thumb): Triggering Slice");
                                }
                            }
                        }



                    }
                }
            }
        }

        // 1. UPDATE VALUE WHILE DRAGGING
        if slider.dragging.is_some() && mouse_button.pressed(MouseButton::Left) {
            if let Some(norm) = rel_pos.normalized {
                let val = if slider.is_vertical { (1.0 - norm.y).clamp(0.0, 1.0) } else { norm.x.clamp(0.0, 1.0) };
                let target = slider.dragging.unwrap();
                match target {
                    RangeSliderThumb::Min => slider.min_value = val.min(slider.max_value - 0.02),
                    RangeSliderThumb::Max => slider.max_value = val.max(slider.min_value + 0.02),
                }
                slicing_settings.dragging_gizmo = Some(match target {
                    RangeSliderThumb::Min => super::super::SlicingGizmoType::Bottom,
                    RangeSliderThumb::Max => super::super::SlicingGizmoType::Top,
                });
                slicing_settings.needs_confirm = false;
            }
        }

        // 2. START DRAGGING
        if *interaction == Interaction::Pressed && slider.dragging.is_none() {
            if let Some(target) = hovered {
                slider.dragging = Some(target);
            } else if let Some(norm) = rel_pos.normalized {
                let val = if slider.is_vertical { 1.0 - norm.y } else { norm.x };
                let dist_min = (val - slider.min_value).abs();
                let dist_max = (val - slider.max_value).abs();
                slider.dragging = Some(if dist_min < dist_max { RangeSliderThumb::Min } else { RangeSliderThumb::Max });
            }
        }

        // 3. RELEASE LOGIC
        if mouse_button.just_released(MouseButton::Left) && slider.dragging.is_some() {
            let target = slider.dragging.unwrap();
            let mut released_inside = false;

            // Check if mouse is over the circle associated with this thumb
            for &track_entity in children.iter() {
                if let Ok(track_children) = node_query.get(track_entity) {
                    for &child in track_children.iter() {
                        if let Ok((_ti, _st, thumb_type, _tt, thumb_children, _tr, thumb_rel_pos)) = thumb_query.get(child) {
                            if *thumb_type == target {
                                if thumb_rel_pos.mouse_over() {
                                    released_inside = true;
                                }
                                for &circle_entity in thumb_children.iter() {
                                    if let Ok((_c, _v, _b, _i, circle_rel_pos)) = circle_query.get(circle_entity) {
                                        if circle_rel_pos.mouse_over() {
                                            released_inside = true;
                                        }
                                    }
                                }
                            }
                        }

                    }
                }
            }

            if released_inside {
                slicing_settings.trigger_slice = true;
                info!("Release INSIDE: Slicing");
            } else {
                slicing_settings.needs_confirm = true;
                info!("Release OUTSIDE: Pending Confirm");
            }
            slider.dragging = None;
        }
    }
}

