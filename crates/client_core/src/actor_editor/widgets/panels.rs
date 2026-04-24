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
    spawn_collapsible_section_ext(parent, font, icon_font, title, is_open, content_bundle, add_content, |_| {});
}

pub fn spawn_collapsible_section_ext<T: Bundle>(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
    title: &str,
    is_open: bool,
    content_bundle: T,
    add_content: impl FnOnce(&mut ChildBuilder),
    add_header_extra: impl FnOnce(&mut ChildBuilder),
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
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                ..default()
            },
            CollapsibleHeader,
        )).with_children(|h| {
            h.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                ..default()
            }).with_children(|left| {
                left.spawn(TextBundle::from_section(
                    if is_open { "\u{f078} " } else { "\u{f054} " },
                    TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7) },
                ));
                left.spawn(TextBundle::from_section(
                    title,
                    TextStyle { font: font.clone(), font_size: 16.0, color: Color::WHITE },
                ));
            });

            h.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            }).with_children(add_header_extra);
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
    mut section_query: Query<(Entity, &mut CollapsibleSection)>,
    mut content_query: Query<(&mut Style, &Parent), With<CollapsibleContent>>,
    header_query: Query<(&Children, &Parent), With<CollapsibleHeader>>,
    container_query: Query<&Children, Without<CollapsibleHeader>>,
    mut text_query: Query<&mut Text>,
) {
    // 1. Handle manual clicks
    for (interaction, parent) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Ok((_, mut section)) = section_query.get_mut(parent.get()) {
                section.is_open = !section.is_open;
            }
        }
    }

    // 2. Sync visual state for any section that changed
    for (section_entity, section) in section_query.iter() {
        // Update content visibility
        for (mut style, parent) in content_query.iter_mut() {
            if parent.get() == section_entity {
                let target_display = if section.is_open { Display::Flex } else { Display::None };
                if style.display != target_display {
                    style.display = target_display;
                }
            }
        }

        // Update header icon
        for (header_children, header_parent) in header_query.iter() {
            if header_parent.get() == section_entity {
                    let target_icon = if section.is_open { "\u{f078} ".to_string() } else { "\u{f054} ".to_string() };
                    
                    // Try old hierarchy first (header_children[0] is Text)
                    if let Ok(mut text) = text_query.get_mut(header_children[0]) {
                        if text.sections[0].value != target_icon {
                            text.sections[0].value = target_icon;
                        }
                    } else if let Ok(container_children) = container_query.get(header_children[0]) {
                        // Try new hierarchy (header_children[0] is Container, container_children[0] is Icon)
                        if let Ok(mut text) = text_query.get_mut(container_children[0]) {
                            if text.sections[0].value != target_icon {
                                text.sections[0].value = target_icon;
                            }
                        }
                    }
                }
            }
    }
}

#[derive(Component, Default)]
pub struct ScrollingList {
    pub position: f32,
}

#[derive(Component)]
pub struct ScrollbarTrack {
    pub target: Entity,
}

#[derive(Component)]
pub struct ScrollbarHandle {
    pub target: Entity,
}

pub fn scroll_system(
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    parent_node_query: Query<(&Node, &GlobalTransform)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };

    for event in mouse_wheel_events.read() {
        for (mut scrolling_list, _style, parent, node) in query.iter_mut() {
            let Ok((parent_node, parent_transform)) = parent_node_query.get(parent.get()) else { continue; };
            
            // Check if cursor is within parent rect
            let parent_size = parent_node.size();
            let parent_pos = parent_transform.translation().truncate();
            let half_size = parent_size / 2.0;
            let min = parent_pos - half_size;
            let max = parent_pos + half_size;
            
            if cursor_position.x < min.x || cursor_position.x > max.x ||
               cursor_position.y < min.y || cursor_position.y > max.y {
                continue;
            }

            let container_height = parent_node.size().y;
            let content_height = node.size().y;
            let max_scroll = (content_height - container_height).max(0.0);
            
            // Allow small buffer to prevent flickering
            if max_scroll > 1.0 {
                scrolling_list.position += event.y * 35.0;
                scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.0);
            } else {
                scrolling_list.position = 0.0;
            }
        }
    }
}

pub fn scrolling_list_sync_system(
    mut query: Query<(&ScrollingList, &mut Style), Changed<ScrollingList>>,
) {
    for (scrolling_list, mut style) in query.iter_mut() {
        style.top = Val::Px(scrolling_list.position);
    }
}

pub fn scrollbar_sync_system(
    scrolling_list_query: Query<(Entity, &ScrollingList, &Node, &Parent)>,
    mut scrollbar_query: Query<(&mut Style, &ScrollbarHandle)>,
    parent_node_query: Query<&Node>,
) {
    for (list_entity, scrolling_list, node, parent) in scrolling_list_query.iter() {
        let Ok(parent_node) = parent_node_query.get(parent.get()) else { continue; };
        let content_height = node.size().y;
        let container_height = parent_node.size().y;
        
        for (mut style, handle) in scrollbar_query.iter_mut() {
            if handle.target == list_entity {
                // Use a slightly larger threshold for visibility to prevent artifacts
                if content_height <= container_height + 5.0 {
                    style.display = Display::None;
                } else {
                    let scroll_percent = (-scrolling_list.position / (content_height - container_height)).clamp(0.0, 1.0);
                    let handle_size_percent = (container_height / content_height).clamp(0.1, 1.0);
                    
                    style.display = Display::Flex;
                    style.height = Val::Percent(handle_size_percent * 100.0);
                    style.top = Val::Percent(scroll_percent * (1.0 - handle_size_percent) * 100.0);
                }
            }
        }
    }
}

pub fn scrollbar_sync_visibility_system(
    scrolling_list_query: Query<(Entity, &Node, &Parent), With<ScrollingList>>,
    mut track_query: Query<(&mut Style, &ScrollbarTrack)>,
    parent_node_query: Query<&Node>,
) {
    for (list_entity, node, parent) in scrolling_list_query.iter() {
        let Ok(parent_node) = parent_node_query.get(parent.get()) else { continue; };
        let content_height = node.size().y;
        let container_height = parent_node.size().y;
        
        for (mut style, track) in track_query.iter_mut() {
            if track.target == list_entity {
                if content_height <= container_height + 5.0 {
                    style.display = Display::None;
                } else {
                    style.display = Display::Flex;
                }
            }
        }
    }
}

pub fn scrollbar_drag_system(
    interaction_query: Query<(&Interaction, &ScrollbarHandle), Changed<Interaction>>,
    mut scrolling_list_query: Query<(&mut ScrollingList, &Node, &Parent)>,
    parent_node_query: Query<(&Node, &GlobalTransform)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut dragging: Local<Option<Entity>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };

    for (interaction, handle) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            *dragging = Some(handle.target);
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        *dragging = None;
    }

    if let Some(target_list_entity) = *dragging {
        if let Ok((mut scrolling_list, node, parent)) = scrolling_list_query.get_mut(target_list_entity) {
            let Ok((parent_node, parent_transform)) = parent_node_query.get(parent.get()) else { return; };
            let parent_size = parent_node.size();
            let parent_pos = parent_transform.translation().truncate();
            let half_size = parent_size / 2.0;
            
            let relative_y = cursor_position.y - (parent_pos.y - half_size.y);
            let scroll_ratio = (relative_y / parent_size.y).clamp(0.0, 1.0);

            let content_height = node.size().y;
            let container_height = parent_size.y;
            if content_height > container_height {
                let max_scroll = content_height - container_height;
                scrolling_list.position = -scroll_ratio * max_scroll;
            }
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
