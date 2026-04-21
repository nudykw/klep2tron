use bevy::prelude::*;
use bevy::input::ButtonState;
use super::{ViewportSettings, ResetCameraEvent};

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

#[derive(Component, Clone, Copy)]
pub enum PanelResizer {
    Left,
    Right,
}

#[derive(Resource)]
pub struct PanelSettings {
    pub left_width: f32,
    pub right_width: f32,
    pub left_visible: bool,
    pub right_visible: bool,
}

impl Default for PanelSettings {
    fn default() -> Self {
        Self {
            left_width: 280.0,
            right_width: 320.0,
            left_visible: true,
            right_visible: true,
        }
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
            Interaction::Pressed => {
                *dragging = Some(*resizer);
                *bg = Color::srgba(0.3, 0.6, 1.0, 0.5).into();
            }
            Interaction::Hovered => {
                *bg = Color::srgba(1.0, 1.0, 1.0, 0.1).into();
            }
            Interaction::None => {
                *bg = Color::srgba(1.0, 1.0, 1.0, 0.02).into();
            }
        }
    }

    for event in mouse_events.read() {
        if event.button == MouseButton::Left && event.state == ButtonState::Released {
            *dragging = None;
        }
    }

    if let Some(side) = *dragging {
        match side {
            PanelResizer::Left => {
                panel_settings.left_width = cursor_position.x.clamp(100.0, 600.0);
            }
            PanelResizer::Right => {
                panel_settings.right_width = (window.width() - cursor_position.x).clamp(100.0, 600.0);
            }
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
pub enum ViewportToggleType {
    Grid,
    Slices,
    Sockets,
    Gizmos,
    Reset,
}

#[derive(Component)]
pub struct ViewportToggleButton(pub ViewportToggleType);

pub fn viewport_button_system(
    mut interaction_query: Query<(&Interaction, &ViewportToggleButton), Changed<Interaction>>,
    mut viewport_settings: ResMut<ViewportSettings>,
    mut reset_events: EventWriter<ResetCameraEvent>,
    mut all_buttons: Query<(&ViewportToggleButton, &mut BackgroundColor)>,
) {
    for (interaction, toggle) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match toggle.0 {
                    ViewportToggleType::Grid => viewport_settings.grid = !viewport_settings.grid,
                    ViewportToggleType::Slices => viewport_settings.slices = !viewport_settings.slices,
                    ViewportToggleType::Sockets => viewport_settings.sockets = !viewport_settings.sockets,
                    ViewportToggleType::Gizmos => viewport_settings.gizmos = !viewport_settings.gizmos,
                    ViewportToggleType::Reset => { reset_events.send(ResetCameraEvent); }
                }
            }
            _ => {}
        }
    }

    // Update visuals based on state
    for (toggle, mut bg) in all_buttons.iter_mut() {
        let active = match toggle.0 {
            ViewportToggleType::Grid => viewport_settings.grid,
            ViewportToggleType::Slices => viewport_settings.slices,
            ViewportToggleType::Sockets => viewport_settings.sockets,
            ViewportToggleType::Gizmos => viewport_settings.gizmos,
            ViewportToggleType::Reset => false,
        };

        if active {
            *bg = Color::srgba(0.3, 0.6, 1.0, 0.8).into();
        } else {
            *bg = Color::srgba(0.2, 0.2, 0.2, 0.9).into();
        }
    }
}

#[derive(Component)]
pub struct Tooltip(pub String);

#[derive(Component)]
pub struct TooltipRoot;

pub fn spawn_tooltip_root(
    commands: &mut Commands,
    font: &Handle<Font>,
    target_camera: Option<Entity>,
) {
    let mut cmd = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(10.0)),
                display: Display::None,
                width: Val::Auto,
                height: Val::Auto,
                ..default()
            },
            background_color: Color::srgba(0.05, 0.05, 0.05, 0.95).into(),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            z_index: ZIndex::Global(100),
            ..default()
        },
        TooltipRoot,
        super::ActorEditorEntity,
    ));

    if let Some(camera) = target_camera {
        cmd.insert(bevy::ui::TargetCamera(camera));
    }

    cmd.with_children(|p| {
        p.spawn(TextBundle::from_section(
            "",
            TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
        ));
    });
}

pub fn tooltip_system(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    interaction_query: Query<(&Interaction, &Tooltip)>,
    mut tooltip_query: Query<(&mut Style, &mut Visibility, &Children), With<TooltipRoot>>,
    mut text_query: Query<&mut Text>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok((mut style, mut visibility, children)) = tooltip_query.get_single_mut() else { return; };

    let mut hovered_text = None;
    for (interaction, tooltip) in interaction_query.iter() {
        if *interaction == Interaction::Hovered {
            hovered_text = Some(tooltip.0.clone());
            break;
        }
    }

    if let Some(text_content) = hovered_text {
        if let Ok(mut text) = text_query.get_mut(children[0]) {
            text.sections[0].value = text_content;
        }
        *visibility = Visibility::Visible;
        style.display = Display::Flex;
        
        // Offset and boundary check
        let x = (cursor_position.x + 15.0).min(window.width() - 150.0);
        let y = (cursor_position.y + 15.0).min(window.height() - 40.0);
        
        style.left = Val::Px(x);
        style.top = Val::Px(y);
    } else {
        *visibility = Visibility::Hidden;
        style.display = Display::None;
    }
}

// --- STATUS BAR ---

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct PolycountText;

#[derive(Component)]
pub struct KeyHintText;

pub fn spawn_status_bar(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(28.0),
            border: UiRect::top(Val::Px(1.0)),
            padding: UiRect::horizontal(Val::Px(15.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        background_color: Color::srgba(0.05, 0.05, 0.05, 0.9).into(),
        border_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
        ..default()
    }).with_children(|p| {
        // Left: Status
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).with_children(|left| {
            left.spawn(TextBundle::from_section(
                "\u{f05a} ",
                TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.3, 0.6, 1.0) },
            ));
            left.spawn((
                TextBundle::from_section(
                    "READY",
                    TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.8, 0.8, 0.8) },
                ),
                StatusText,
            ));
        });

        // Center: Key Hints
        p.spawn((
            TextBundle::from_section(
                "TAB: Switch Mode | G: Toggle Grid | R: Reset View",
                TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) },
            ),
            KeyHintText,
        ));

        // Right: Stats
        p.spawn((
            TextBundle::from_section(
                "POLYS: 0",
                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.7, 0.7, 0.7) },
            ),
            PolycountText,
        ));
    });
}

// --- TOASTS ---

#[derive(Component)]
pub struct ToastContainer;

#[derive(Component)]
pub struct ToastTimer(pub Timer);

pub fn spawn_toast_container(commands: &mut Commands, target_camera: Option<Entity>) -> Entity {
    let mut cmd = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                right: Val::Px(20.0),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::End,
                ..default()
            },
            z_index: ZIndex::Global(110),
            ..default()
        },
        ToastContainer,
        super::ActorEditorEntity,
    ));

    if let Some(camera) = target_camera {
        cmd.insert(bevy::ui::TargetCamera(camera));
    }

    cmd.id()
}

pub fn spawn_toast_item(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
    message: &str,
    toast_type: super::ToastType,
) {
    let (icon, color) = match toast_type {
        super::ToastType::Info => ("\u{f05a}", Color::srgb(0.3, 0.6, 1.0)),
        super::ToastType::Success => ("\u{f058}", Color::srgb(0.3, 0.8, 0.3)),
        super::ToastType::Error => ("\u{f071}", Color::srgb(0.8, 0.3, 0.3)),
    };

    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(250.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgba(0.1, 0.1, 0.1, 0.95).into(),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            border_color: color.with_alpha(0.3).into(),
            ..default()
        },
        ToastTimer(Timer::from_seconds(4.0, TimerMode::Once)),
    )).with_children(|p| {
        p.spawn(TextBundle::from_section(
            format!("{} ", icon),
            TextStyle { font: icon_font.clone(), font_size: 18.0, color },
        ));
        p.spawn(TextBundle::from_section(
            message,
            TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
        ));
    });
}

// --- MODALS ---

#[derive(Component)]
pub struct ModalOverlay;

#[derive(Component)]
pub struct ConfirmModalButton(pub super::EditorAction);

#[derive(Component)]
pub struct CancelModalButton;

pub fn spawn_confirmation_modal(
    commands: &mut Commands,
    font: &Handle<Font>,
    _icon_font: &Handle<Font>,
    title: &str,
    message: &str,
    action: super::EditorAction,
    target_camera: Option<Entity>,
) {
    let mut cmd = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(),
            z_index: ZIndex::Global(200),
            ..default()
        },
        ModalOverlay,
        super::ActorEditorEntity,
    ));

    if let Some(camera) = target_camera {
        cmd.insert(bevy::ui::TargetCamera(camera));
    }

    cmd.with_children(|p| {
        p.spawn(NodeBundle {
            style: Style {
                width: Val::Px(400.0),
                padding: UiRect::all(Val::Px(25.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::srgba(0.15, 0.15, 0.15, 1.0).into(),
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        }).with_children(|modal| {
            // Title
            modal.spawn(TextBundle::from_section(
                title.to_uppercase(),
                TextStyle { font: font.clone(), font_size: 20.0, color: Color::WHITE },
            ));

            // Message
            modal.spawn(NodeBundle {
                style: Style {
                    margin: UiRect::vertical(Val::Px(20.0)),
                    ..default()
                },
                ..default()
            }).with_children(|m| {
                m.spawn(TextBundle::from_section(
                    message,
                    TextStyle { font: font.clone(), font_size: 15.0, color: Color::srgb(0.8, 0.8, 0.8) },
                ));
            });

            // Buttons
            modal.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::End,
                    ..default()
                },
                ..default()
            }).with_children(|btns| {
                // Cancel
                btns.spawn((
                    ButtonBundle {
                        style: Style {
                            padding: UiRect::horizontal(Val::Px(20.0)),
                            height: Val::Px(35.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::right(Val::Px(10.0)),
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    CancelModalButton,
                )).with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "CANCEL",
                        TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                    ));
                });

                // Confirm
                btns.spawn((
                    ButtonBundle {
                        style: Style {
                            padding: UiRect::horizontal(Val::Px(20.0)),
                            height: Val::Px(35.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(0.8, 0.2, 0.2, 0.8).into(),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    ConfirmModalButton(action),
                )).with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "CONFIRM",
                        TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                    ));
                });
            });
        });
    });
}
