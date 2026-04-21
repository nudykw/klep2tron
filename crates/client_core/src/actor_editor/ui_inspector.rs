use bevy::prelude::*;
use super::widgets::{spawn_collapsible_section, spawn_slider};

#[derive(Component)]
pub struct InspectorPanel;

#[derive(Component)]
pub struct SocketSearchInput;

#[derive(Component)]
pub struct SocketListItem {
    pub name: String,
}

#[derive(Component)]
pub struct MaterialColorPreview;

#[derive(Resource, Default)]
pub struct SocketFilter(pub String);

#[derive(Resource, Default)]
pub struct SelectedSocket(pub Option<Entity>);

#[derive(Component)]
pub enum TransformAxis {
    X, Y, Z
}

pub fn setup_inspector(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        },
        InspectorPanel,
    )).with_children(|p| {
        // --- MATERIALS SECTION ---
        spawn_collapsible_section(
            p,
            font,
            icon_font,
            "MATERIALS",
            true,
            (),
            |content| {
                content.spawn(TextBundle::from_section(
                    "Color",
                    TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                ));
                super::widgets::spawn_color_picker(content, font, Color::srgb(0.7, 0.7, 0.7), false);

                content.spawn(TextBundle::from_section(
                    "Metallic",
                    TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                ));
                spawn_slider(content, 0.5);

                content.spawn(TextBundle::from_section(
                    "Roughness",
                    TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
                ));
                spawn_slider(content, 0.8);
            }
        );

        // --- SOCKETS SECTION ---
        spawn_collapsible_section(
            p,
            font,
            icon_font,
            "SOCKETS",
            true,
            (),
            |content| {
                content.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(30.0),
                            margin: UiRect::bottom(Val::Px(10.0)),
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    SocketSearchInput,
                )).with_children(|search| {
                    search.spawn(TextBundle::from_section(
                        "\u{f002} ", // fa-search
                        TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.5, 0.5, 0.5) },
                    ));
                    search.spawn(TextBundle::from_section(
                        "Search sockets...",
                        TextStyle { font: font.clone(), font_size: 14.0, color: Color::srgb(0.5, 0.5, 0.5) },
                    ));
                });

                let sockets = vec!["head", "hand_l", "hand_r", "back", "foot_l", "foot_r"];
                for name in sockets {
                    content.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(25.0),
                                margin: UiRect::bottom(Val::Px(2.0)),
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::srgba(1.0, 1.0, 1.0, 0.02).into(),
                            ..default()
                        },
                        SocketListItem { name: name.to_string() },
                    )).with_children(|item| {
                        item.spawn(TextBundle::from_section(
                            name,
                            TextStyle { font: font.clone(), font_size: 14.0, color: Color::srgb(0.9, 0.9, 0.9) },
                        ));
                    });
                }
                
                content.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        margin: UiRect::top(Val::Px(15.0)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                }).with_children(|row| {
                    for (axis, label) in [(TransformAxis::X, "X"), (TransformAxis::Y, "Y"), (TransformAxis::Z, "Z")] {
                        row.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(75.0),
                                    height: Val::Px(25.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            axis,
                        )).with_children(|box_| {
                            box_.spawn(TextBundle::from_section(
                                format!("{}: {:.2}", label, 0.0),
                                TextStyle { font: font.clone(), font_size: 12.0, color: Color::WHITE },
                            ));
                        });
                    }
                });
            }
        );
    });
}

pub fn socket_filter_system(
    filter: Res<SocketFilter>,
    mut query: Query<(&SocketListItem, &mut Visibility, &mut BackgroundColor)>,
) {
    if !filter.is_changed() { return; }
    let search = filter.0.to_lowercase();
    for (item, mut visibility, mut bg) in query.iter_mut() {
        if search.is_empty() {
            *visibility = Visibility::Visible;
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.02).into();
        } else if item.name.to_lowercase().contains(&search) {
            *visibility = Visibility::Visible;
            *bg = Color::srgba(0.3, 0.6, 1.0, 0.2).into();
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn socket_transform_update_system(
    selected: Res<SelectedSocket>,
    socket_query: Query<&Transform>,
    mut axis_text_query: Query<(&TransformAxis, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    let Some(entity) = selected.0 else { return; };
    let Ok(transform) = socket_query.get(entity) else { return; };
    for (axis, children) in axis_text_query.iter_mut() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(*child) {
                let (val, label) = match axis {
                    TransformAxis::X => (transform.translation.x, "X"),
                    TransformAxis::Y => (transform.translation.y, "Y"),
                    TransformAxis::Z => (transform.translation.z, "Z"),
                };
                text.sections[0].value = format!("{}: {:.2}", label, val);
            }
        }
    }
}
