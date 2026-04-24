use bevy::prelude::*;
use crate::{GameState, GraphicsSettings};
use super::types::*;

pub fn spawn_menu_button(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    text: &str,
    value: Option<String>,
    index: usize,
    item_type: MenuItemType,
    action: MenuAction,
    tooltip: Option<String>,
    is_disabled: bool,
) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(460.0),
                height: Val::Px(50.0),
                margin: UiRect::vertical(Val::Px(5.0)),
                padding: UiRect::horizontal(Val::Px(20.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            background_color: if is_disabled { Color::srgba(0.1, 0.1, 0.1, 0.3).into() } else { Color::srgba(1.0, 1.0, 1.0, 0.05).into() },
            border_color: Color::NONE.into(),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..default()
        },
        MenuItem { index, item_type, action, tooltip, is_disabled },
    )).with_children(|p| {
        let text_color = if is_disabled { Color::srgb(0.4, 0.4, 0.4) } else { Color::srgb(0.9, 0.9, 0.9) };

        p.spawn(TextBundle::from_section(
            text,
            TextStyle { font: font.clone(), font_size: 26.0, color: text_color },
        ).with_text_justify(JustifyText::Left));

        if let Some(val) = value {
            p.spawn(TextBundle::from_section(
                format!("< {} >", val),
                TextStyle { font: font.clone(), font_size: 26.0, color: text_color },
            ).with_text_justify(JustifyText::Right));
        } else if item_type == MenuItemType::Submenu {
            p.spawn(TextBundle::from_section(
                ">",
                TextStyle { font: font.clone(), font_size: 26.0, color: text_color },
            ).with_text_justify(JustifyText::Right));
        }
    });
}

pub fn setup_menu(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    game_state: Res<State<GameState>>,
    exit_confirm: Res<ExitConfirmationActive>,
    camera_query: Query<Entity, With<Camera2d>>,
    mut next_menu_state: ResMut<NextState<MenuSubState>>,
) {
    if *game_state.get() == GameState::Menu {
        next_menu_state.set(MenuSubState::Main);
    }
    
    if camera_query.is_empty() {
        commands.spawn((
            Camera3dBundle {
                camera: Camera { order: -1, ..default() },
                transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(1.0, 0.0, 0.0), Vec3::Y),
                ..default()
            },
            MenuEntity,
        ));

        commands.spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 10,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            },
            MenuEntity,
        ));
    }
    
    let font = asset_server.load("fonts/Roboto-Regular.ttf");

    commands.spawn((NodeBundle {
        style: Style {
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            display: Display::Flex, flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center, justify_content: JustifyContent::FlexStart,
            position_type: PositionType::Absolute,
            padding: UiRect::top(Val::Px(60.0)),
            ..default()
        },
        background_color: if exit_confirm.0 { Color::srgba(0.0, 0.0, 0.05, 0.6).into() } else { Color::NONE.into() },
        ..default()
    }, MenuEntity, MenuItemRoot)).with_children(|p| {
        p.spawn((TextBundle::from_section(
            "Klep2tron",
            TextStyle { font: font.clone(), font_size: 70.0, color: Color::WHITE },
        ).with_style(Style { 
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        }), MenuTitleDisplay));

        p.spawn((NodeBundle {
            style: Style {
                width: Val::Px(500.0),
                height: Val::Px(420.0), 
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                overflow: Overflow::clip(),
                margin: UiRect::top(Val::Px(20.0)),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::vertical(Val::Px(10.0)),
                ..default()
            },
            background_color: Color::srgba(0.05, 0.05, 0.15, 0.4).into(),
            border_color: Color::srgba(0.3, 0.5, 1.0, 0.2).into(),
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        }, MenuViewport));
    });

    commands.spawn((NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        z_index: ZIndex::Global(100),
        ..default()
    }, MenuEntity)).with_children(|p| {
        p.spawn((TextBundle::from_section(
            "",
            TextStyle { font_size: 20.0, color: Color::srgb(0.9, 0.9, 0.6), ..default() },
        ).with_style(Style { margin: UiRect::bottom(Val::Px(10.0)), ..default() }), TooltipDisplay));

        p.spawn((TextBundle::from_section(
            "",
            TextStyle { font_size: 18.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() },
        ), InputHintFooter));
    });
}

pub fn menu_item_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu_state: Res<State<MenuSubState>>,
    settings: Res<GraphicsSettings>,
    pending: Res<PendingGraphicsSettings>,
    confirmation: Res<ConfirmationData>,
    extra_buttons: Res<ExtraMenuButtons>,
    gpu_list: Res<crate::settings::GpuList>,
    root_query: Query<(Entity, &Children), With<MenuItemRoot>>,
    mut title_query: Query<&mut Text, With<MenuTitleDisplay>>,
    viewport_query: Query<Entity, With<MenuViewport>>,
    viewport_children_query: Query<&Children, With<MenuViewport>>,
    selection_memory: Res<MenuSelectionMemory>,
    overlay_query: Query<Entity, With<crate::ConfirmationOverlay>>,
    focus_query: Query<Entity, With<MenuFocus>>,
) {
    let Ok((root, _root_children)) = root_query.get_single() else { return; };
    let viewport_entity = viewport_query.get_single().ok();
    
    let is_empty = if let Some(v) = viewport_entity {
        if let Ok(children) = viewport_children_query.get(v) {
            children.is_empty()
        } else {
            true
        }
    } else {
        true
    };

    if !menu_state.is_changed() && !settings.is_changed() && !pending.is_changed() && !is_empty { return; }
    
    let is_confirmation = *menu_state.get() == MenuSubState::Confirmation;

    if !is_confirmation {
        if let Ok(mut text) = title_query.get_single_mut() {
            text.sections[0].value = match *menu_state.get() {
                MenuSubState::Main => "Klep2tron".to_string(),
                MenuSubState::Settings => "Settings".to_string(),
                MenuSubState::Advanced => "Advanced".to_string(),
                _ => text.sections[0].value.clone(),
            };
        }
    }
    
    if !is_confirmation {
        for entity in overlay_query.iter() {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn_recursive();
            }
        }
    } else {
        if commands.get_entity(root).is_some() {
            commands.entity(root).remove::<MenuContainer>();
        }
        for focus_entity in focus_query.iter() {
            commands.entity(focus_entity).remove::<MenuFocus>();
        }
    }

    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let mut new_container: Option<MenuContainer> = None;
    
    if let Some(v) = viewport_entity {
        if !is_confirmation {
            if commands.get_entity(v).is_some() {
                commands.entity(v).despawn_descendants();
            }
            
            commands.entity(v).with_children(|parent| {
                parent.spawn((NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(20.0),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    ..default()
                }, MenuScrollContainer)).with_children(|scroll_p| {
                    match *menu_state.get() {
                        MenuSubState::Main => {
                            spawn_menu_button(scroll_p, &font, "START GAME", None, 0, MenuItemType::Action, MenuAction::StartGame, Some("Start a new game session".to_string()), false);
                            let mut count = 1;
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                spawn_menu_button(scroll_p, &font, "ACTOR EDITOR", None, count, MenuItemType::Action, MenuAction::OpenActorEditor, Some("Modular character editor".to_string()), false);
                                count += 1;
                            }
                            for (_i, (name, action)) in extra_buttons.buttons.iter().enumerate() {
                                spawn_menu_button(scroll_p, &font, name, None, count, MenuItemType::Action, action.clone(), None, false);
                                count += 1;
                            }
                            spawn_menu_button(scroll_p, &font, "SETTINGS", None, count, MenuItemType::Submenu, MenuAction::OpenSettings, Some("Configure graphics and input".to_string()), false);
                            count += 1;
                            spawn_menu_button(scroll_p, &font, "EXIT", None, count, MenuItemType::Action, MenuAction::Exit, Some("Quit to desktop".to_string()), false);
                            count += 1;
                            new_container = Some(MenuContainer { current_selection: 0, items_count: count });
                        },
                        MenuSubState::Settings => {
                            spawn_menu_button(scroll_p, &font, "BACK", None, 0, MenuItemType::Action, MenuAction::Back, None, false);
                            spawn_menu_button(scroll_p, &font, "QUALITY", Some(format!("{:?}", pending.quality_level)), 1, MenuItemType::Toggle, MenuAction::NextQuality, Some("Global quality preset".to_string()), false);
                            spawn_menu_button(scroll_p, &font, "UPSCALING", Some(format!("{:?}", pending.upscaling)), 2, MenuItemType::Toggle, MenuAction::NextUpscaling, Some("FSR 1.0 or TAA".to_string()), false);
                            spawn_menu_button(scroll_p, &font, "VSYNC", Some(if pending.vsync { "ON" } else { "OFF" }.to_string()), 3, MenuItemType::Toggle, MenuAction::ToggleVSync, None, false);
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                spawn_menu_button(scroll_p, &font, "MODE", Some(format!("{:?}", pending.window_mode)), 4, MenuItemType::Toggle, MenuAction::NextWindowMode, None, false);
                                spawn_menu_button(scroll_p, &font, "ADVANCED", None, 5, MenuItemType::Submenu, MenuAction::OpenAdvanced, Some("GPU selection and more".to_string()), false);
                                spawn_menu_button(scroll_p, &font, "APPLY", None, 6, MenuItemType::Action, MenuAction::ApplySettings, None, false);
                            }
                            let count = if cfg!(target_arch = "wasm32") { 4 } else { 7 };
                            new_container = Some(MenuContainer { current_selection: 0, items_count: count });
                        },
                        MenuSubState::Advanced => {
                            spawn_menu_button(scroll_p, &font, "BACK", None, 0, MenuItemType::Action, MenuAction::Back, None, false);
                            spawn_menu_button(scroll_p, &font, "RUN BENCHMARK", None, 1, MenuItemType::Action, MenuAction::RunBenchmark, Some("Test hardware performance".to_string()), false);
                            let gpu_val = pending.selected_gpu.clone().unwrap_or_else(|| {
                                if gpu_list.names.is_empty() { "Detecting...".to_string() } else { gpu_list.names[0].clone() }
                            });
                            spawn_menu_button(scroll_p, &font, "GPU", Some(gpu_val), 2, MenuItemType::Toggle, MenuAction::NextGpu, Some("Select graphics hardware".to_string()), false);
                            spawn_menu_button(scroll_p, &font, "SHADOWS", Some(format!("{:?}", pending.shadow_quality)), 3, MenuItemType::Toggle, MenuAction::NextShadowQuality, None, false);
                            spawn_menu_button(scroll_p, &font, "FOG", Some(format!("{:?}", pending.fog_quality)), 4, MenuItemType::Toggle, MenuAction::NextFog, None, false);
                            spawn_menu_button(scroll_p, &font, "BLOOM", Some(if pending.bloom { "ON" } else { "OFF" }.to_string()), 5, MenuItemType::Toggle, MenuAction::ToggleBloom, None, false);
                            spawn_menu_button(scroll_p, &font, "SSAO", Some(format!("{:?}", pending.ssao)), 6, MenuItemType::Toggle, MenuAction::NextSsao, None, false);
                            spawn_menu_button(scroll_p, &font, "SHADOW RES", Some(pending.shadow_resolution.to_string()), 7, MenuItemType::Toggle, MenuAction::NextShadowRes, None, false);
                            spawn_menu_button(scroll_p, &font, "FPS LIMIT", Some(if pending.fps_limit_enabled { "ON" } else { "OFF" }.to_string()), 8, MenuItemType::Toggle, MenuAction::ToggleFpsLimit, None, false);
                            spawn_menu_button(scroll_p, &font, "FPS VALUE", Some(pending.fps_limit.to_string()), 9, MenuItemType::Toggle, MenuAction::NextFpsLimit, None, !pending.fps_limit_enabled);
                            new_container = Some(MenuContainer { current_selection: 0, items_count: 10 });
                        },
                        _ => {}
                    }
                });
            });
        }
    }

    if is_confirmation {
        let mut count = 2;
        if confirmation.has_cancel { count = 3; }

        let overlay_id = commands.spawn((NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
            z_index: ZIndex::Global(1000),
            ..default()
        }, MenuEntity, crate::ConfirmationOverlay)).with_children(|p| {
            p.spawn((NodeBundle {
                style: Style {
                    width: Val::Px(550.0), height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(30.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                background_color: Color::srgba(0.05, 0.08, 0.15, 0.95).into(),
                border_color: Color::srgba(0.3, 0.5, 1.0, 0.5).into(),
                border_radius: BorderRadius::all(Val::Px(15.0)),
                ..default()
            },)).with_children(|inner| {
                inner.spawn(TextBundle::from_section(
                    &confirmation.message,
                    TextStyle { font: font.clone(), font_size: 32.0, color: Color::WHITE },
                ).with_style(Style { margin: UiRect::bottom(Val::Px(40.0)), ..default() }).with_text_justify(JustifyText::Center));

                spawn_menu_button(inner, &font, "YES", None, 0, MenuItemType::Action, MenuAction::ConfirmYes, None, false);
                spawn_menu_button(inner, &font, "NO", None, 1, MenuItemType::Action, MenuAction::ConfirmNo, None, false);
                if confirmation.has_cancel {
                    spawn_menu_button(inner, &font, "CANCEL", None, 2, MenuItemType::Action, MenuAction::ConfirmCancel, None, false);
                }
            });
        }).id();
        
        commands.entity(overlay_id).insert(MenuContainer { current_selection: 0, items_count: count });
    }

    if let Some(container) = new_container {
        let mut final_container = container;
        if let Some(&saved_idx) = selection_memory.selections.get(menu_state.get()) {
            if saved_idx < final_container.items_count {
                final_container.current_selection = saved_idx;
            }
        }
        commands.entity(root).insert(final_container);
    }
}

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in query.iter() { commands.entity(entity).despawn_recursive(); }
}

pub fn sync_pending_settings(
    menu_state: Res<State<MenuSubState>>,
    settings: Res<GraphicsSettings>,
    mut pending: ResMut<PendingGraphicsSettings>,
) {
    if menu_state.is_changed() && *menu_state.get() == MenuSubState::Settings {
        **pending = (*settings).clone();
    }
}
