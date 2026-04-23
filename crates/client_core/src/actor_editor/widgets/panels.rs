use bevy::prelude::*;
use bevy::input::ButtonState;
use super::super::PanelResizer;

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
