use bevy::prelude::*;
use bevy::input::ButtonState;
use super::{ViewportSettings, ResetCameraEvent, PanelResizer};

#[derive(Component)]
pub struct CollapsibleSection {
    pub is_open: bool,
}

#[derive(Component)]
pub struct CollapsibleHeader;

#[derive(Component)]
pub struct CollapsibleContent;

pub fn spawn_collapsible_section<T: Bundle>(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
    title: &str,
    is_open: bool,
    content_bundle: T,
    add_content: impl FnOnce(&mut ChildBuilder),
) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            ..default()
        },
        CollapsibleSection { is_open },
    )).with_children(|p| {
        p.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                ..default()
            },
            CollapsibleHeader,
        )).with_children(|h| {
            h.spawn(TextBundle::from_section(
                if is_open { "\u{f078} " } else { "\u{f054} " },
                TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7) },
            ));
            h.spawn(TextBundle::from_section(
                title,
                TextStyle { font: font.clone(), font_size: 16.0, color: Color::WHITE },
            ));
        });

        p.spawn((
            content_bundle,
            CollapsibleContent,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    display: if is_open { Display::Flex } else { Display::None },
                    ..default()
                },
                ..default()
            },
        )).with_children(add_content);
    });
}

pub fn collapsible_system(
    mut interaction_query: Query<(&Interaction, &Parent), (Changed<Interaction>, With<CollapsibleHeader>)>,
    mut section_query: Query<&mut CollapsibleSection>,
    mut content_query: Query<(&mut Style, &Parent), With<CollapsibleContent>>,
    header_text_query: Query<(&Children, &Parent), With<CollapsibleHeader>>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, parent) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut section) = section_query.get_mut(parent.get()) {
                section.is_open = !section.is_open;
                for (mut style, content_parent) in content_query.iter_mut() {
                    if content_parent.get() == parent.get() {
                        style.display = if section.is_open { Display::Flex } else { Display::None };
                    }
                }
                for (children, header_parent) in header_text_query.iter() {
                    if header_parent.get() == parent.get() {
                        if let Ok(mut text) = text_query.get_mut(children[0]) {
                            text.sections[0].value = if section.is_open { "\u{f078} ".to_string() } else { "\u{f054} ".to_string() };
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct ScrollingList {
    pub position: f32,
}

pub fn scroll_system(
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    parent_query: Query<&Node>,
) {
    for event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, node) in query.iter_mut() {
            let Ok(parent_node) = parent_query.get(parent.get()) else { continue; };
            let content_height = node.size().y;
            let container_height = parent_node.size().y;
            if content_height <= container_height {
                scrolling_list.position = 0.0;
                style.top = Val::Px(0.0);
                continue;
            }
            let max_scroll = content_height - container_height;
            scrolling_list.position += event.y * 20.0;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.0);
            style.top = Val::Px(scrolling_list.position);
        }
    }
}

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
                    background_color: Color::NONE.into(), // Background is transparent, icon is visible
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
    slicing_settings: Res<super::SlicingSettings>,
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

        // Sync visual styles and tooltips (Always, to reflect auto-init or logic changes)
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
                        // Update tooltip text with current value
                        let base = match thumb { RangeSliderThumb::Min => "Bottom Cut (Engine)", RangeSliderThumb::Max => "Top Cut (Head)" };
                        tooltip.0 = format!("{}: {:.0}%", base, pct);
                    }
                }
            }
        }

        if *interaction != Interaction::None {
            // ... interaction logic handled by hovered_thumb check ...
        }

        if *interaction == Interaction::Pressed {
            let rect = node.size();
            let pos = transform.translation().truncate();
            let val = if slider.is_vertical {
                // Corrected coordinate system for vertical UI
                let local_y = (pos.y + rect.y / 2.0) - cursor_position.y;
                (local_y / rect.y).clamp(0.0, 1.0)
            } else {
                let local_x = (cursor_position.x - (pos.x - rect.x / 2.0)) / rect.x;
                local_x.clamp(0.0, 1.0)
            };

            // If we're not dragging yet, pick the closest thumb
            if slider.dragging.is_none() {
                let dist_min = (val - slider.min_value).abs();
                let dist_max = (val - slider.max_value).abs();
                if dist_min < dist_max {
                    slider.dragging = Some(RangeSliderThumb::Min);
                } else {
                    slider.dragging = Some(RangeSliderThumb::Max);
                }
            }

            // Move the dragged thumb
            if let Some(target) = slider.dragging {
                match target {
                    RangeSliderThumb::Min => slider.min_value = val.min(slider.max_value - 0.02),
                    RangeSliderThumb::Max => slider.max_value = val.max(slider.min_value + 0.02),
                }
            }
        } else {
            // Reset dragging when mouse is released
            slider.dragging = None;
        }
    }
}

#[derive(Component)]
// PanelResizer is now a component in mod.rs

#[derive(Resource)]
pub struct PanelSettings {
    pub left_width: f32,
    pub right_width: f32,
    pub left_visible: bool,
    pub right_visible: bool,
}

impl Default for PanelSettings {
    fn default() -> Self {
        Self { left_width: 280.0, right_width: 320.0, left_visible: true, right_visible: true, }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ResizablePanel(pub PanelResizer);

pub fn panel_resize_system(
    mut mouse_events: EventReader<bevy::input::mouse::MouseButtonInput>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut resizer_query: Query<(&Interaction, &PanelResizer, &mut BackgroundColor)>,
    mut panel_settings: ResMut<PanelSettings>,
    mut dragging: Local<Option<PanelResizer>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    for (interaction, resizer, mut bg) in resizer_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => { *dragging = Some(*resizer); *bg = Color::srgba(0.3, 0.6, 1.0, 0.5).into(); }
            Interaction::Hovered => { *bg = Color::srgba(1.0, 1.0, 1.0, 0.1).into(); }
            Interaction::None => { *bg = Color::srgba(1.0, 1.0, 1.0, 0.02).into(); }
        }
    }
    for event in mouse_events.read() { if event.button == MouseButton::Left && event.state == ButtonState::Released { *dragging = None; } }
    if let Some(side) = *dragging {
        match side {
            PanelResizer::Left => { panel_settings.left_width = cursor_position.x.clamp(100.0, 600.0); }
            PanelResizer::Right => { panel_settings.right_width = (window.width() - cursor_position.x).clamp(100.0, 600.0); }
        }
    }
}

pub fn update_panel_style_system(
    panel_settings: Res<PanelSettings>,
    mut query: Query<(&mut Style, &ResizablePanel)>,
) {
    if !panel_settings.is_changed() { return; }
    for (mut style, panel) in query.iter_mut() {
        match panel.0 {
            PanelResizer::Left => {
                style.width = Val::Px(if panel_settings.left_visible { panel_settings.left_width } else { 0.0 });
                style.display = if panel_settings.left_visible { Display::Flex } else { Display::None };
            }
            PanelResizer::Right => {
                style.width = Val::Px(if panel_settings.right_visible { panel_settings.right_width } else { 0.0 });
                style.display = if panel_settings.right_visible { Display::Flex } else { Display::None };
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct PanelToggle(pub PanelResizer);

pub fn panel_toggle_system(
    mut interaction_query: Query<(&Interaction, &PanelToggle), Changed<Interaction>>,
    mut panel_settings: ResMut<PanelSettings>,
) {
    for (interaction, toggle) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match toggle.0 {
                PanelResizer::Left => panel_settings.left_visible = !panel_settings.left_visible,
                PanelResizer::Right => panel_settings.right_visible = !panel_settings.right_visible,
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum ViewportToggleType { Grid, Slices, Sockets, Gizmos, Reset, }

#[derive(Component)]
pub struct ViewportToggleButton(pub ViewportToggleType);

pub fn viewport_button_system(
    _interaction_query: Query<(&Interaction, &ViewportToggleButton), Changed<Interaction>>,
    viewport_settings: Res<ViewportSettings>,
    _reset_events: EventWriter<ResetCameraEvent>,
    mut all_buttons: Query<(&ViewportToggleButton, &mut BackgroundColor)>,
) {
    for (toggle, mut bg) in all_buttons.iter_mut() {
        let active = match toggle.0 {
            ViewportToggleType::Grid => viewport_settings.grid,
            ViewportToggleType::Slices => viewport_settings.slices,
            ViewportToggleType::Sockets => viewport_settings.sockets,
            ViewportToggleType::Gizmos => viewport_settings.gizmos,
            ViewportToggleType::Reset => false,
        };
        if active { *bg = Color::srgba(0.3, 0.6, 1.0, 0.8).into(); } else { *bg = Color::srgba(0.2, 0.2, 0.2, 0.9).into(); }
    }
}

#[derive(Component)]
pub struct Tooltip(pub String);
#[derive(Component)]
pub struct TooltipRoot;
pub fn spawn_tooltip_root(commands: &mut Commands, font: &Handle<Font>, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((NodeBundle { style: Style { position_type: PositionType::Absolute, padding: UiRect::all(Val::Px(10.0)), display: Display::None, width: Val::Auto, height: Val::Auto, ..default() }, background_color: Color::srgba(0.05, 0.05, 0.05, 0.95).into(), border_radius: BorderRadius::all(Val::Px(6.0)), z_index: ZIndex::Global(100), ..default() }, TooltipRoot, super::ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::TargetCamera(camera)); }
    cmd.with_children(|p| { p.spawn(TextBundle::from_section("", TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE }, )); });
}

pub fn tooltip_system(window_query: Query<&Window, With<bevy::window::PrimaryWindow>>, interaction_query: Query<(&Interaction, &Tooltip)>, mut tooltip_query: Query<(&mut Style, &mut Visibility, &Children), With<TooltipRoot>>, mut text_query: Query<&mut Text>, ) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok((mut style, mut visibility, children)) = tooltip_query.get_single_mut() else { return; };
    let mut hovered_text = None;
    for (interaction, tooltip) in interaction_query.iter() { if *interaction == Interaction::Hovered { hovered_text = Some(tooltip.0.clone()); break; } }
    if let Some(text_content) = hovered_text {
        if let Ok(mut text) = text_query.get_mut(children[0]) { text.sections[0].value = text_content; }
        *visibility = Visibility::Visible; style.display = Display::Flex;
        let x = (cursor_position.x + 15.0).min(window.width() - 150.0);
        let y = (cursor_position.y + 15.0).min(window.height() - 40.0);
        style.left = Val::Px(x); style.top = Val::Px(y);
    } else { *visibility = Visibility::Hidden; style.display = Display::None; }
}

#[derive(Component)]
pub struct StatusText;
#[derive(Component)]
pub struct PolycountText;
#[derive(Component)]
pub struct KeyHintText;
pub fn spawn_status_bar(parent: &mut ChildBuilder, font: &Handle<Font>, icon_font: &Handle<Font>) {
    parent.spawn(NodeBundle { style: Style { width: Val::Percent(100.0), height: Val::Px(28.0), border: UiRect::top(Val::Px(1.0)), padding: UiRect::horizontal(Val::Px(15.0)), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, ..default() }, background_color: Color::srgba(0.05, 0.05, 0.05, 0.9).into(), border_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(), ..default() }).with_children(|p| {
        p.spawn((NodeBundle { style: Style { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }, ..default() }, Interaction::default(), Tooltip("Current Editor State".to_string()), )).with_children(|left| {
            left.spawn(TextBundle::from_section("\u{f05a} ", TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.3, 0.6, 1.0) }));
            left.spawn((TextBundle::from_section("READY", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.7, 0.7, 0.7) }), StatusText));
        });
        p.spawn((NodeBundle { style: Style { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }, ..default() }, Interaction::default(), Tooltip("Keyboard Shortcuts & Gizmo Legend".to_string()), )).with_children(|mid| {
            mid.spawn((TextBundle::from_sections(vec![
                TextSection::new("TAB: Mode | G: Grid | R: Reset | ", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) }),
                TextSection::new("X", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(1.0, 0.3, 0.3) }),
                TextSection::new(":R ", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) }),
                TextSection::new("Y", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.3, 1.0, 0.3) }),
                TextSection::new(":G ", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) }),
                TextSection::new("Z", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.4, 0.4, 1.0) }),
                TextSection::new(":B", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) }),
            ]), KeyHintText));
        });
        p.spawn((NodeBundle { style: Style { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }, ..default() }, Interaction::default(), Tooltip("Total Scene Complexity".to_string()), )).with_children(|right| {
            right.spawn(TextBundle::from_section("\u{f1b2} ", TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7) }));
            right.spawn((TextBundle::from_section("POLYS: 0", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.7, 0.7, 0.7) }), PolycountText));
        });
    });
}

#[derive(Component)]
pub struct ToastContainer;
#[derive(Component)]
pub struct ToastTimer(pub Timer);
pub fn spawn_toast_container(commands: &mut Commands, target_camera: Option<Entity>) -> Entity {
    let mut cmd = commands.spawn((NodeBundle { style: Style { position_type: PositionType::Absolute, bottom: Val::Px(40.0), right: Val::Px(20.0), flex_direction: FlexDirection::ColumnReverse, align_items: AlignItems::End, ..default() }, z_index: ZIndex::Global(110), ..default() }, ToastContainer, super::ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::TargetCamera(camera)); }
    cmd.id()
}
pub fn spawn_toast_item(parent: &mut ChildBuilder, font: &Handle<Font>, icon_font: &Handle<Font>, message: &str, toast_type: super::ToastType) {
    let (icon, color) = match toast_type { super::ToastType::Info => ("\u{f05a}", Color::srgb(0.3, 0.6, 1.0)), super::ToastType::Success => ("\u{f058}", Color::srgb(0.3, 0.8, 0.3)), super::ToastType::Error => ("\u{f071}", Color::srgb(0.8, 0.3, 0.3)), };
    parent.spawn((NodeBundle { style: Style { width: Val::Px(250.0), margin: UiRect::bottom(Val::Px(10.0)), padding: UiRect::all(Val::Px(12.0)), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }, background_color: Color::srgba(0.1, 0.1, 0.1, 0.95).into(), border_radius: BorderRadius::all(Val::Px(8.0)), border_color: color.with_alpha(0.3).into(), ..default() }, ToastTimer(Timer::from_seconds(4.0, TimerMode::Once)), )).with_children(|p| {
        p.spawn(TextBundle::from_section(format!("{} ", icon), TextStyle { font: icon_font.clone(), font_size: 18.0, color }));
        p.spawn(TextBundle::from_section(message, TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE }));
    });
}

#[derive(Component)]
pub struct ModalOverlay;
#[derive(Component)]
pub struct ConfirmModalButton(pub super::EditorAction);
#[derive(Component)]
pub struct CancelModalButton;
pub fn spawn_confirmation_modal(commands: &mut Commands, font: &Handle<Font>, _icon_font: &Handle<Font>, title: &str, message: &str, action: super::EditorAction, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((NodeBundle { style: Style { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }, background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(), z_index: ZIndex::Global(200), ..default() }, ModalOverlay, super::ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::TargetCamera(camera)); }
    cmd.with_children(|p| {
        p.spawn(NodeBundle { style: Style { width: Val::Px(400.0), padding: UiRect::all(Val::Px(25.0)), flex_direction: FlexDirection::Column, ..default() }, background_color: Color::srgba(0.15, 0.15, 0.15, 1.0).into(), border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }).with_children(|modal| {
            modal.spawn(TextBundle::from_section(title.to_uppercase(), TextStyle { font: font.clone(), font_size: 20.0, color: Color::WHITE }));
            modal.spawn(NodeBundle { style: Style { margin: UiRect::vertical(Val::Px(20.0)), ..default() }, ..default() }).with_children(|m| { m.spawn(TextBundle::from_section(message, TextStyle { font: font.clone(), font_size: 15.0, color: Color::srgb(0.8, 0.8, 0.8) })); });
            modal.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Row, justify_content: JustifyContent::End, ..default() }, ..default() }).with_children(|btns| {
                btns.spawn((ButtonBundle { style: Style { padding: UiRect::horizontal(Val::Px(20.0)), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::right(Val::Px(10.0)), ..default() }, background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, CancelModalButton, )).with_children(|btn| { btn.spawn(TextBundle::from_section("CANCEL", TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE })); });
                btns.spawn((ButtonBundle { style: Style { padding: UiRect::horizontal(Val::Px(20.0)), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, background_color: Color::srgba(0.8, 0.2, 0.2, 0.8).into(), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, ConfirmModalButton(action), )).with_children(|btn| { btn.spawn(TextBundle::from_section("CONFIRM", TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE })); });
            });
        });
    });
}

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

#[derive(Component)]
pub struct ProgressBarFill;
#[derive(Component)]
pub struct LoadingOverlay;
#[derive(Component)]
pub struct ProgressBarText;
pub fn spawn_progress_bar(parent: &mut ChildBuilder, font: &Handle<Font>) {
    parent.spawn(NodeBundle { style: Style { width: Val::Px(300.0), height: Val::Px(40.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, ..default() }, ..default() }).with_children(|p| {
        p.spawn(NodeBundle { style: Style { width: Val::Percent(100.0), height: Val::Px(8.0), ..default() }, background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }).with_children(|bg| {
            bg.spawn((NodeBundle { style: Style { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() }, background_color: Color::srgb(0.3, 0.6, 1.0).into(), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, ProgressBarFill, ));
        });
        p.spawn((TextBundle::from_section("0%", TextStyle { font: font.clone(), font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7) }).with_style(Style { margin: UiRect::top(Val::Px(8.0)), align_self: AlignSelf::Center, ..default() }), ProgressBarText, ));
    });
}
pub fn spawn_loading_overlay(commands: &mut Commands, font: &Handle<Font>, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((NodeBundle { style: Style { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, align_items: AlignItems::Center, justify_content: JustifyContent::Center, flex_direction: FlexDirection::Column, ..default() }, background_color: Color::srgba(0.0, 0.0, 0.0, 0.85).into(), z_index: ZIndex::Global(300), ..default() }, LoadingOverlay, super::ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::TargetCamera(camera)); }
    cmd.with_children(|p| {
        p.spawn(TextBundle::from_section("IMPORTING MODEL", TextStyle { font: font.clone(), font_size: 24.0, color: Color::WHITE }).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default() }));
        spawn_progress_bar(p, font);
    });
}

#[derive(Component)]
pub struct SlicerLockButton;
#[derive(Component)]
pub struct SlicerContainer;

pub fn spawn_viewport_slicer(parent: &mut ChildBuilder, icon_font: &Handle<Font>, initial_min: f32, initial_max: f32) {
    parent.spawn((NodeBundle { 
        style: Style { 
            position_type: PositionType::Absolute, 
            left: Val::Px(20.0), 
            top: Val::Px(150.0), 
            height: Val::Px(400.0), 
            width: Val::Px(40.0), 
            flex_direction: FlexDirection::Column, 
            align_items: AlignItems::Center, 
            padding: UiRect::vertical(Val::Px(10.0)), 
            ..default() 
        }, 
        background_color: Color::srgba(0.1, 0.1, 0.1, 0.6).into(),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default() 
    }, SlicerContainer, )).with_children(|p| {
        p.spawn((ButtonBundle { 
            style: Style { 
                width: Val::Px(30.0), 
                height: Val::Px(30.0), 
                justify_content: JustifyContent::Center, 
                align_items: AlignItems::Center, 
                margin: UiRect::bottom(Val::Px(10.0)), 
                ..default() 
            }, 
            background_color: Color::srgba(0.2, 0.2, 0.2, 0.9).into(), 
            border_radius: BorderRadius::all(Val::Px(6.0)), 
            ..default() 
        }, SlicerLockButton, Tooltip("Lock/Unlock Slicer (L)".to_string()), )).with_children(|btn| {
            btn.spawn(TextBundle::from_section("\u{f023}", TextStyle { font: icon_font.clone(), font_size: 16.0, color: Color::WHITE }, ));
        });
        spawn_vertical_range_slider(p, icon_font, initial_min, initial_max);
    });
}
