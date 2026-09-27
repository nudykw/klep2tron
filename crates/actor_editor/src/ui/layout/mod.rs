use bevy::prelude::*;
use client_core::ui::widgets::*;
use crate::{ActorEditorEntity, ActorEditorBackButton, PanelResizer};
use crate::widgets::{ScrollingList, ScrollbarTrack, ScrollbarHandle, ResizablePanel, PanelToggle, PanelSettings, Tooltip, spawn_tooltip_root, ViewportToggleType, ViewportToggleButton, spawn_viewport_slicer};

pub mod camera;
pub mod gizmo_legend;

pub fn setup_actor_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    panel_settings: Res<PanelSettings>,
    vfx_presets: Res<crate::vfx_assets::VfxPresets>,
    vfx_registry: Res<crate::vfx_assets::VfxRegistry>,
) {
    // 3D Camera and Lighting
    let main_camera_entity = camera::spawn_actor_editor_cameras(&mut commands);
    camera::spawn_actor_editor_lighting(&mut commands, main_camera_entity);

    // Gizmo Axes Legend
    gizmo_legend::spawn_gizmo_legend(&mut commands, &mut meshes, &mut materials);

    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");

    commands.insert_resource(crate::EditorFonts {
        regular: font.clone(),
        icon: icon_font.clone(),
    });

    // Spawn Tooltip Root
    spawn_tooltip_root(&mut commands, &font, Some(main_camera_entity));

    // Root UI Node (Vertical Column)
    commands.spawn((
        UiNode { node: Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            }, ..default() },
        ActorEditorEntity,
        bevy::ui::UiTargetCamera(main_camera_entity),
    )).with_children(|root| {
        // --- MAIN AREA ---
        root.spawn(UiNode { node: Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                ..default()
            }, ..default() }).with_children(|parent| {
            // --- LEFT SIDEBAR ---
            parent.spawn((
                UiNode { node: Node {
                        width: Val::Px(panel_settings.left_width),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::right(Val::Px(1.5)),
                        overflow: Overflow::clip(),
                        ..default()
                    }, background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.75)), border_color: BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)), ..default() },
                ResizablePanel(PanelResizer::Left),
                Interaction::default(),
            )).with_children(|p| {
                // --- WRAPPER ---
                let mut scroll_id = None;
                p.spawn(UiNode { node: Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        overflow: Overflow::clip(),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    }, ..default() }).with_children(|wrapper| {
                    scroll_id = Some(wrapper.spawn((
                        UiNode { node: Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect { left: Val::Px(15.0), right: Val::Px(15.0), top: Val::Px(15.0), bottom: Val::Px(0.0) },
                                position_type: PositionType::Absolute,
                                height: Val::Auto,
                                flex_shrink: 0.0,
                                ..default()
                            }, ..default() },
                        ScrollingList { position: 0.0 },
                        Interaction::default(),
                    )).with_children(|scroll_p| {
                        scroll_p.spawn(ui_text("PROJECT", &font.clone(), 20.0, Color::srgb(0.7, 0.7, 0.7)));
                        crate::ui_project::setup_project_panel(scroll_p, &font, &icon_font);
                        
                        // Spacer at the bottom
                        scroll_p.spawn(UiNode { node: Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(60.0),
                                ..default()
                            }, ..default() });
                    }).id());
                });
                let scroll_id = scroll_id.unwrap();

                // --- SCROLLBAR ---
                p.spawn((
                    UiNode { node: Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(2.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                            width: Val::Px(4.0),
                            ..default()
                        }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() },
                    ScrollbarTrack { target: scroll_id },
                )).with_children(|track| {
                    track.spawn((
                        UiNode { node: Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(20.0),
                                border_radius: BorderRadius::all(Val::Px(2.0)), ..default()
                            }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.2)), ..default() },
                        ScrollbarHandle { target: scroll_id },
                        Interaction::default(),
                    ));
                });
            });

            parent.spawn((
                (Button, UiNode { node: Node {
                        width: Val::Px(8.0),
                        height: Val::Percent(100.0),
                        ..default()
                    }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.02)), z_index: ZIndex(10), ..default() }),
                PanelResizer::Left,
                Tooltip("Resize Project Panel".to_string()),
            ));

            // --- CENTER VIEWPORT SPACE ---
            parent.spawn(UiNode { node: Node {
                    flex_grow: 1.0,
                    ..default()
                }, ..default() }).with_children(|p| {
                // Header
                p.spawn(UiNode { node: Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    }, ..default() }).with_children(|header| {
                    header.spawn(ui_text("ACTOR EDITOR", &font.clone(), 28.0, Color::WHITE));
                });

                // --- TOP TOOLBAR ---
                p.spawn(UiNode { node: Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(70.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    }, ..default() }).with_children(|toolbar| {
                    toolbar.spawn(UiNode { node: Node {
                            padding: UiRect::all(Val::Px(4.0)),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(8.0)), ..default()
                        }, background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)), ..default() }).with_children(|btns| {
                        spawn_viewport_button(btns, ViewportToggleType::Grid, "\u{f00a}", "Toggle Grid (G)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Slices, "\u{f121}", "Toggle Slices (S)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Sockets, "\u{f1e0}", "Toggle Sockets (K)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Gizmos, "\u{f047}", "Toggle Gizmos (Z)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Xray, "\u{f06e}", "Toggle X-Ray (X)", &icon_font);
                        spawn_viewport_button(btns, ViewportToggleType::Reset, "\u{f021}", "Reset Camera (R)", &icon_font);
                        btns.spawn(UiNode { node: Node { width: Val::Px(2.0), height: Val::Px(20.0), margin: UiRect::horizontal(Val::Px(8.0)), ..default() }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)), ..default() });
                        spawn_undo_redo_button(btns, true, &icon_font);
                        spawn_undo_redo_button(btns, false, &icon_font);
                    });
                });

                // VIEWPORT SLICER
                spawn_viewport_slicer(p, &icon_font, 0.0, 1.0);

                // Toggle Sidebars
                p.spawn((
                    (Button, UiNode { node: Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(15.0),
                            top: Val::Px(15.0),
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)), ..default()
                        }, background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)), ..default() }),
                    PanelToggle(PanelResizer::Left),
                    Tooltip("Toggle Project Panel".to_string()),
                )).with_children(|btn| {
                    btn.spawn(ui_text("\u{f0c9}", &icon_font.clone(), 20.0, Color::WHITE));
                });

                p.spawn((
                    (Button, UiNode { node: Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(15.0),
                            top: Val::Px(15.0),
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)), ..default()
                        }, background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)), ..default() }),
                    PanelToggle(PanelResizer::Right),
                    Tooltip("Toggle Inspector Panel".to_string()),
                )).with_children(|btn| {
                    btn.spawn(ui_text("\u{f0c9}", &icon_font.clone(), 20.0, Color::WHITE));
                });
            });

            parent.spawn((
                (Button, UiNode { node: Node {
                        width: Val::Px(8.0),
                        height: Val::Percent(100.0),
                        ..default()
                    }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.02)), z_index: ZIndex(10), ..default() }),
                PanelResizer::Right,
                Tooltip("Resize Inspector Panel".to_string()),
            ));

            // --- RIGHT SIDEBAR ---
            parent.spawn((
                UiNode { node: Node {
                        width: Val::Px(panel_settings.right_width),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::left(Val::Px(1.5)),
                        overflow: Overflow::clip(),
                        ..default()
                    }, background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.75)), border_color: BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)), ..default() },
                ResizablePanel(PanelResizer::Right),
                Interaction::default(),
            )).with_children(|p| {
                // --- WRAPPER ---
                let mut scroll_id = None;
                p.spawn(UiNode { node: Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        overflow: Overflow::clip(),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    }, ..default() }).with_children(|wrapper| {
                    scroll_id = Some(wrapper.spawn((
                        UiNode { node: Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect { left: Val::Px(15.0), right: Val::Px(15.0), top: Val::Px(15.0), bottom: Val::Px(0.0) },
                                position_type: PositionType::Absolute,
                                height: Val::Auto,
                                flex_shrink: 0.0,
                                ..default()
                            }, ..default() },
                        ScrollingList { position: 0.0 },
                        Interaction::default(),
                    )).with_children(|scroll_p| {
                        scroll_p.spawn(ui_text("INSPECTOR", &font.clone(), 20.0, Color::srgb(0.7, 0.7, 0.7)));
                        crate::ui::inspector::setup_inspector(scroll_p, &font, &icon_font, &vfx_presets, &vfx_registry);
                        
                        // Spacer at the bottom
                        scroll_p.spawn(UiNode { node: Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(60.0),
                                ..default()
                            }, ..default() });
                    }).id());
                });
                let scroll_id = scroll_id.unwrap();

                // --- SCROLLBAR ---
                p.spawn((
                    UiNode { node: Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(2.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                            width: Val::Px(4.0),
                            ..default()
                        }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() },
                    ScrollbarTrack { target: scroll_id },
                )).with_children(|track| {
                    track.spawn((
                        UiNode { node: Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(20.0),
                                border_radius: BorderRadius::all(Val::Px(2.0)), ..default()
                            }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.2)), ..default() },
                        ScrollbarHandle { target: scroll_id },
                        Interaction::default(),
                    ));
                });
            });
            
            parent.spawn((
                (Button, UiNode { node: Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(20.0),
                        right: Val::Px(20.0),
                        width: Val::Px(80.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(6.0)), ..default()
                    }, background_color: BackgroundColor(Color::srgba(0.3, 0.1, 0.1, 0.8)), ..default() }),
                ActorEditorBackButton,
                Tooltip("Back to Main Menu".to_string()),
            )).with_children(|p| {
                p.spawn(ui_text("BACK", &font.clone(), 18.0, Color::WHITE));
            });
        });

        crate::widgets::spawn_status_bar(root, &font, &icon_font);
    });

    crate::widgets::spawn_toast_container(&mut commands, Some(main_camera_entity));
    crate::widgets::spawn_loading_overlay(&mut commands, &font, Some(main_camera_entity));
}

pub fn cleanup_actor_editor(
    mut commands: Commands,
    query: Query<Entity, With<ActorEditorEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn_viewport_button(
    parent: &mut ChildSpawnerCommands,
    toggle_type: ViewportToggleType,
    icon: &str,
    tooltip: &str,
    icon_font: &Handle<Font>,
) {
    parent.spawn((
        (Button, UiNode { node: Node {
                width: Val::Px(36.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::horizontal(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)), ..default()
            }, background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)), ..default() }),
        ViewportToggleButton(toggle_type),
        Tooltip(tooltip.to_string()),
    )).with_children(|btn| {
        btn.spawn(ui_text(icon, &icon_font.clone(), 18.0, Color::WHITE));
    });
}

fn spawn_undo_redo_button(
    parent: &mut ChildSpawnerCommands,
    is_undo: bool,
    icon_font: &Handle<Font>,
) {
    let mut builder = parent.spawn((
        (Button, UiNode { node: Node {
                width: Val::Px(36.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::horizontal(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)), ..default()
            }, background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)), ..default() }),
    ));

    if is_undo {
        builder.insert((crate::UndoButton, Tooltip("Undo (Ctrl+Z)".to_string())));
        builder.with_children(|btn| {
            btn.spawn(ui_text("\u{f0e2}", &icon_font.clone(), 18.0, Color::WHITE));
        });
    } else {
        builder.insert((crate::RedoButton, Tooltip("Redo (Ctrl+Y / Ctrl+Shift+Z)".to_string())));
        builder.with_children(|btn| {
            btn.spawn(ui_text("\u{f01e}", &icon_font.clone(), 18.0, Color::WHITE));
        });
    }
}
