use bevy::prelude::*;
use crate::actor_editor::{ActorEditorEntity, ActorEditorBackButton, PanelResizer};
use crate::actor_editor::widgets::{ScrollingList, ScrollbarTrack, ScrollbarHandle, ResizablePanel, PanelToggle, PanelSettings, Tooltip, spawn_tooltip_root, ViewportToggleType, ViewportToggleButton, spawn_viewport_slicer};

pub mod camera;
pub mod gizmo_legend;

pub fn setup_actor_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    panel_settings: Res<PanelSettings>,
) {
    // 3D Camera and Lighting
    let main_camera_entity = camera::spawn_actor_editor_cameras(&mut commands);
    camera::spawn_actor_editor_lighting(&mut commands, main_camera_entity);

    // Gizmo Axes Legend
    gizmo_legend::spawn_gizmo_legend(&mut commands, &mut meshes, &mut materials);

    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");

    commands.insert_resource(crate::actor_editor::EditorFonts {
        regular: font.clone(),
        icon: icon_font.clone(),
    });

    // Spawn Tooltip Root
    spawn_tooltip_root(&mut commands, &font, Some(main_camera_entity));

    // Root UI Node (Vertical Column)
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            focus_policy: bevy::ui::FocusPolicy::Pass,
            ..default()
        },
        ActorEditorEntity,
        bevy::ui::TargetCamera(main_camera_entity),
    )).with_children(|root| {
        // --- MAIN AREA ---
        root.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            focus_policy: bevy::ui::FocusPolicy::Pass,
            ..default()
        }).with_children(|parent| {
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
                Interaction::default(),
            )).with_children(|p| {
                // --- WRAPPER ---
                let mut scroll_id = None;
                p.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        overflow: Overflow::clip(),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    ..default()
                }).with_children(|wrapper| {
                    scroll_id = Some(wrapper.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect { left: Val::Px(15.0), right: Val::Px(15.0), top: Val::Px(15.0), bottom: Val::Px(0.0) },
                                position_type: PositionType::Absolute,
                                height: Val::Auto,
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ..default()
                        },
                        ScrollingList { position: 0.0 },
                        Interaction::default(),
                    )).with_children(|scroll_p| {
                        scroll_p.spawn(TextBundle::from_section(
                            "PROJECT",
                            TextStyle { font: font.clone(), font_size: 20.0, color: Color::srgb(0.7, 0.7, 0.7) },
                        ));
                        crate::actor_editor::ui_project::setup_project_panel(scroll_p, &font, &icon_font);
                        
                        // Spacer at the bottom
                        scroll_p.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(60.0),
                                ..default()
                            },
                            ..default()
                        });
                    }).id());
                });
                let scroll_id = scroll_id.unwrap();

                // --- SCROLLBAR ---
                p.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px(2.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                            width: Val::Px(4.0),
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                        ..default()
                    },
                    ScrollbarTrack { target: scroll_id },
                )).with_children(|track| {
                    track.spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(20.0),
                                ..default()
                            },
                            background_color: Color::srgba(1.0, 1.0, 1.0, 0.2).into(),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            ..default()
                        },
                        ScrollbarHandle { target: scroll_id },
                        Interaction::default(),
                    ));
                });
            });

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
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..default()
            }).with_children(|p| {
                // Header
                p.spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                }).with_children(|header| {
                    header.spawn(TextBundle::from_section(
                        "ACTOR EDITOR",
                        TextStyle { font: font.clone(), font_size: 28.0, color: Color::WHITE },
                    ));
                });

                // --- TOP TOOLBAR ---
                p.spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(70.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                }).with_children(|toolbar| {
                    toolbar.spawn(NodeBundle {
                        style: Style {
                            padding: UiRect::all(Val::Px(4.0)),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        focus_policy: bevy::ui::FocusPolicy::Pass,
                        ..default()
                    }).with_children(|btns| {
                        spawn_viewport_button(btns, ViewportToggleType::Grid, "\u{f00a}", "Toggle Grid (G)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Slices, "\u{f121}", "Toggle Slices (S)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Sockets, "\u{f1e0}", "Toggle Sockets (K)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Gizmos, "\u{f047}", "Toggle Gizmos (Z)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Xray, "\u{f06e}", "Toggle X-Ray (X)", &icon_font);
                        btns.spawn(NodeBundle {
                            style: Style { width: Val::Px(2.0), height: Val::Px(20.0), margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
                            background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                            ..default()
                        });
                        spawn_viewport_button(btns, ViewportToggleType::Reset, "\u{f01e}", "Reset Camera (R)", &icon_font);
                    });
                });

                // VIEWPORT SLICER
                spawn_viewport_slicer(p, &icon_font, 0.0, 1.0);

                // Toggle Sidebars
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
                Interaction::default(),
            )).with_children(|p| {
                // --- WRAPPER ---
                let mut scroll_id = None;
                p.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        overflow: Overflow::clip(),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    ..default()
                }).with_children(|wrapper| {
                    scroll_id = Some(wrapper.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect { left: Val::Px(15.0), right: Val::Px(15.0), top: Val::Px(15.0), bottom: Val::Px(0.0) },
                                position_type: PositionType::Absolute,
                                height: Val::Auto,
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ..default()
                        },
                        ScrollingList { position: 0.0 },
                        Interaction::default(),
                    )).with_children(|scroll_p| {
                        scroll_p.spawn(TextBundle::from_section(
                            "INSPECTOR",
                            TextStyle { font: font.clone(), font_size: 20.0, color: Color::srgb(0.7, 0.7, 0.7) },
                        ));
                        crate::actor_editor::ui::inspector::setup_inspector(scroll_p, &font, &icon_font);
                        
                        // Spacer at the bottom
                        scroll_p.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(60.0),
                                ..default()
                            },
                            ..default()
                        });
                    }).id());
                });
                let scroll_id = scroll_id.unwrap();

                // --- SCROLLBAR ---
                p.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px(2.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                            width: Val::Px(4.0),
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                        ..default()
                    },
                    ScrollbarTrack { target: scroll_id },
                )).with_children(|track| {
                    track.spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(20.0),
                                ..default()
                            },
                            background_color: Color::srgba(1.0, 1.0, 1.0, 0.2).into(),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            ..default()
                        },
                        ScrollbarHandle { target: scroll_id },
                        Interaction::default(),
                    ));
                });
            });
            
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

        crate::actor_editor::widgets::spawn_status_bar(root, &font, &icon_font);
    });

    crate::actor_editor::widgets::spawn_toast_container(&mut commands, Some(main_camera_entity));
    crate::actor_editor::widgets::spawn_loading_overlay(&mut commands, &font, Some(main_camera_entity));
}

pub fn cleanup_actor_editor(
    mut commands: Commands,
    query: Query<Entity, With<ActorEditorEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_viewport_button(
    parent: &mut ChildBuilder,
    toggle_type: ViewportToggleType,
    icon: &str,
    tooltip: &str,
    icon_font: &Handle<Font>,
) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(36.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::horizontal(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::srgba(0.2, 0.2, 0.2, 0.9).into(),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        ViewportToggleButton(toggle_type),
        Tooltip(tooltip.to_string()),
    )).with_children(|btn| {
        btn.spawn(TextBundle::from_section(
            icon,
            TextStyle { font: icon_font.clone(), font_size: 18.0, color: Color::WHITE },
        ));
    });
}
