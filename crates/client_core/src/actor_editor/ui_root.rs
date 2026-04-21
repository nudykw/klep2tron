use bevy::prelude::*;
use super::{ActorEditorEntity, ActorEditorBackButton, MainEditorCamera, GizmoCamera, GizmoEntity, GIZMO_LAYER};
use super::widgets::{ScrollingList, ResizablePanel, PanelResizer, PanelToggle, PanelSettings, Tooltip, spawn_tooltip_root, ViewportToggleType, ViewportToggleButton};

pub fn setup_actor_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    panel_settings: Res<PanelSettings>,
) {
    // 3D Camera
    // 3D Main Camera
    let main_camera_entity = commands.spawn((
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
        MainEditorCamera,
    )).id();

    // Gizmo Camera (Sub-view)
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 10,
                viewport: Some(bevy::render::camera::Viewport {
                    physical_position: UVec2::new(20, 20),
                    physical_size: UVec2::new(120, 120),
                    depth: 0.0..1.0,
                }),
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera_3d: Camera3d::default(),
            transform: Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
        GizmoCamera,
        GIZMO_LAYER,
    ));

    // Spawn Gizmo Axes
    let mesh_handle = asset_server.add(Mesh::from(Cuboid::new(0.02, 0.02, 0.8)));
    
    // X - Red
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle.clone(),
            material: asset_server.add(StandardMaterial { base_color: Color::srgb(1.0, 0.2, 0.2), unlit: true, ..default() }),
            transform: Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                        .with_translation(Vec3::X * 0.4),
            ..default()
        },
        ActorEditorEntity,
        GizmoEntity,
        super::EditorHelper,
        GIZMO_LAYER,
    ));

    let font_bold = asset_server.load("fonts/Roboto-Regular.ttf");

    // Labels
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("X", TextStyle { font: font_bold.clone(), font_size: 48.0, color: Color::srgb(1.0, 0.4, 0.4) }),
            transform: Transform::from_translation(Vec3::X * 0.6).with_scale(Vec3::splat(0.005)),
            ..default()
        },
        ActorEditorEntity,
        super::EditorHelper,
        super::GizmoLabel,
        GIZMO_LAYER,
    ));
    // Y - Green
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle.clone(),
            material: asset_server.add(StandardMaterial { base_color: Color::srgb(0.2, 1.0, 0.2), unlit: true, ..default() }),
            transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                        .with_translation(Vec3::Y * 0.4),
            ..default()
        },
        ActorEditorEntity,
        GizmoEntity,
        super::EditorHelper,
        GIZMO_LAYER,
    ));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("Y", TextStyle { font: font_bold.clone(), font_size: 48.0, color: Color::srgb(0.4, 1.0, 0.4) }),
            transform: Transform::from_translation(Vec3::Y * 0.6).with_scale(Vec3::splat(0.005)),
            ..default()
        },
        ActorEditorEntity,
        super::EditorHelper,
        super::GizmoLabel,
        GIZMO_LAYER,
    ));
    // Z - Blue
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle.clone(),
            material: asset_server.add(StandardMaterial { base_color: Color::srgb(0.2, 0.2, 1.0), unlit: true, ..default() }),
            transform: Transform::from_translation(Vec3::Z * 0.4),
            ..default()
        },
        ActorEditorEntity,
        GizmoEntity,
        super::EditorHelper,
        GIZMO_LAYER,
    ));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("Z", TextStyle { font: font_bold.clone(), font_size: 48.0, color: Color::srgb(0.4, 0.4, 1.0) }),
            transform: Transform::from_translation(Vec3::Z * 0.6).with_scale(Vec3::splat(0.005)),
            ..default()
        },
        ActorEditorEntity,
        super::EditorHelper,
        super::GizmoLabel,
        GIZMO_LAYER,
    ));

    // --- 3-POINT LIGHTING ---
    // Key Light (Main light from front-top-right)
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 25000.0, // Increased
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    // Fill Light (Softer light from the opposite side to soften shadows)
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 12000.0, // Increased
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(-5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    // Back/Rim Light (Light from behind to create highlights on edges)
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 15000.0, // Increased
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 5.0, -8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    // Camera "Headlamp" (Point light attached to the main camera)
    commands.entity(main_camera_entity).with_children(|parent| {
        parent.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 80000.0, // Reduced from 500k
                range: 15.0,
                shadows_enabled: false,
                ..default()
            },
            // Offset slightly to the top-right of camera to create subtle shadows
            transform: Transform::from_xyz(0.8, 0.8, 0.0),
            ..default()
        });
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0, // Reduced from 800
    });

    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");

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
            ..default()
        },
        ActorEditorEntity,
        bevy::ui::TargetCamera(main_camera_entity),
    )).with_children(|root| {
        // --- MAIN AREA (SIDEBARS + VIEWPORT) ---
        root.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                ..default()
            },
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
                focus_policy: bevy::ui::FocusPolicy::Pass,
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
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                }).with_children(|header| {
                    header.spawn(NodeBundle {
                        style: Style {
                            padding: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                        focus_policy: bevy::ui::FocusPolicy::Pass,
                        ..default()
                    }).with_children(|inner| {
                        inner.spawn(TextBundle::from_section(
                            "ACTOR EDITOR",
                            TextStyle { font: font.clone(), font_size: 28.0, color: Color::WHITE },
                        ));
                    });
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
                        // This node has icons, but it's a container.
                        focus_policy: bevy::ui::FocusPolicy::Pass,
                        ..default()
                    }).with_children(|btns| {
                        spawn_viewport_button(btns, ViewportToggleType::Grid, "\u{f00a}", "Toggle Grid (G)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Slices, "\u{f121}", "Toggle Slices (S)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Sockets, "\u{f1e0}", "Toggle Sockets (K)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Gizmos, "\u{f047}", "Toggle Gizmos (Z)", &icon_font);
                        
                        // Separator
                        btns.spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(2.0),
                                height: Val::Px(20.0),
                                margin: UiRect::horizontal(Val::Px(8.0)),
                                ..default()
                            },
                            background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                            ..default()
                        });

                        spawn_viewport_button(btns, ViewportToggleType::Reset, "\u{f01e}", "Reset Camera (R)", &icon_font);
                    });
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

        // --- BOTTOM STATUS BAR ---
        super::widgets::spawn_status_bar(root, &font, &icon_font);
    });

    // Spawn Toast Container
    super::widgets::spawn_toast_container(&mut commands, Some(main_camera_entity));

    // Spawn Loading Overlay
    super::widgets::spawn_loading_overlay(&mut commands, &font, Some(main_camera_entity));
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
