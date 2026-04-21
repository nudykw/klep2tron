use bevy::prelude::*;
use super::{ActorEditorEntity, ActorEditorBackButton};
use super::widgets::{ScrollingList, ResizablePanel, PanelResizer, PanelToggle, PanelSettings, Tooltip, spawn_tooltip_root};

pub fn setup_actor_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    panel_settings: Res<PanelSettings>,
) {
    // 3D Camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 5,
                clear_color: Color::BLACK.into(),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    // Light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(2.0, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");

    // Spawn Tooltip Root
    spawn_tooltip_root(&mut commands, &font);

    // Root UI Node
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        },
        ActorEditorEntity,
    )).with_children(|parent| {
        // --- LEFT SIDEBAR ---
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(panel_settings.left_width),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::right(Val::Px(1.5)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                background_color: Color::srgba(0.1, 0.1, 0.1, 0.75).into(),
                border_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                ..default()
            },
            ResizablePanel(PanelResizer::Left),
        )).with_children(|p| {
            p.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(15.0)),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    ..default()
                },
                ScrollingList { position: 0.0 },
            )).with_children(|scroll_p| {
                scroll_p.spawn(TextBundle::from_section(
                    "PROJECT",
                    TextStyle { font: font.clone(), font_size: 20.0, color: Color::srgb(0.7, 0.7, 0.7) },
                ));
                super::ui_project::setup_project_panel(scroll_p, &font, &icon_font);
            });
        });

        // LEFT RESIZER HANDLE
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(8.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.02).into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            PanelResizer::Left,
            Tooltip("Resize Project Panel".to_string()),
        ));

        // --- CENTER VIEWPORT SPACE ---
        parent.spawn(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                ..default()
            },
            ..default()
        }).with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            }).with_children(|header| {
                header.spawn(TextBundle::from_section(
                    "ACTOR EDITOR",
                    TextStyle { font: font.clone(), font_size: 28.0, color: Color::WHITE },
                ));
            });

            // Toggle Visibility Buttons
            p.spawn((
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(15.0),
                        top: Val::Px(15.0),
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.2, 0.2, 0.2, 0.9).into(),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                PanelToggle(PanelResizer::Left),
                Tooltip("Toggle Project Panel".to_string()),
            )).with_children(|btn| {
                btn.spawn(TextBundle::from_section(
                    "\u{f0c9}",
                    TextStyle { font: icon_font.clone(), font_size: 20.0, color: Color::WHITE },
                ));
            });

            p.spawn((
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        right: Val::Px(15.0),
                        top: Val::Px(15.0),
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.2, 0.2, 0.2, 0.9).into(),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                PanelToggle(PanelResizer::Right),
                Tooltip("Toggle Inspector Panel".to_string()),
            )).with_children(|btn| {
                btn.spawn(TextBundle::from_section(
                    "\u{f0c9}",
                    TextStyle { font: icon_font.clone(), font_size: 20.0, color: Color::WHITE },
                ));
            });
        });

        // RIGHT RESIZER HANDLE
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(8.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.02).into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            PanelResizer::Right,
            Tooltip("Resize Inspector Panel".to_string()),
        ));

        // --- RIGHT SIDEBAR ---
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(panel_settings.right_width),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::left(Val::Px(1.5)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                background_color: Color::srgba(0.1, 0.1, 0.1, 0.75).into(),
                border_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                ..default()
            },
            ResizablePanel(PanelResizer::Right),
        )).with_children(|p| {
            p.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(15.0)),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    ..default()
                },
                ScrollingList { position: 0.0 },
            )).with_children(|scroll_p| {
                scroll_p.spawn(TextBundle::from_section(
                    "INSPECTOR",
                    TextStyle { font: font.clone(), font_size: 20.0, color: Color::srgb(0.7, 0.7, 0.7) },
                ));
                super::ui_inspector::setup_inspector(scroll_p, &font, &icon_font);
            });
        });
        
        // Back Button
        parent.spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    right: Val::Px(20.0),
                    width: Val::Px(80.0),
                    height: Val::Px(35.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.3, 0.1, 0.1, 0.8).into(),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            ActorEditorBackButton,
            Tooltip("Back to Main Menu".to_string()),
        )).with_children(|p| {
            p.spawn(TextBundle::from_section(
                "BACK",
                TextStyle { font: font.clone(), font_size: 18.0, color: Color::WHITE },
            ));
        });
    });
}

pub fn cleanup_actor_editor(
    mut commands: Commands,
    query: Query<Entity, With<ActorEditorEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
