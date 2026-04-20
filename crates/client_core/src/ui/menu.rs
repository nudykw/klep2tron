use bevy::prelude::*;
use bevy::input::gamepad::GamepadEvent;
use crate::{GameState, GraphicsSettings, save_settings, MyWindowMode, UpscalingMode, QualityLevel, EditorMode, ExitConfirmationActive};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuItemType {
    Action,
    Submenu,
    Toggle,
    Slider,
}

#[derive(Clone, Debug, PartialEq, Component)]
pub enum MenuAction {
    StartGame,
    StartEditor,
    Exit,
    Navigate(String), 
    Back,
    OpenSettings,
    ApplySettings,
    ToggleVSync,
    NextWindowMode, PrevWindowMode,
    NextUpscaling, PrevUpscaling,
    NextQuality, PrevQuality,
    SetResolution(u32, u32),
    ConfirmYes,
    ConfirmNo,
    ConfirmCancel,
    None,
}

#[derive(Resource, Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum InputDevice {
    #[default]
    Keyboard,
    Gamepad,
    Mouse,
    Touch,
}

#[derive(States, Default, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum MenuSubState {
    #[default]
    Main,
    Settings,
    Confirmation,
}

#[derive(Component)]
pub struct MenuContainer {
    pub current_selection: usize,
    pub items_count: usize,
}

#[derive(Component)]
pub struct MenuItem {
    pub index: usize,
    pub item_type: MenuItemType,
    pub action: MenuAction,
    pub tooltip: Option<String>,
}

#[derive(Component)]
pub struct MenuFocus; 

#[derive(Component)]
pub struct MenuTooltip {
    pub timer: Timer,
}

#[derive(Component)]
pub struct InputHintFooter;

#[derive(Resource, Default)]
pub struct ExtraMenuButtons {
    pub buttons: Vec<(String, MenuAction)>,
}

#[derive(Resource, Default)]
pub struct MenuSelectionMemory {
    pub selections: std::collections::HashMap<MenuSubState, usize>,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PendingGraphicsSettings(pub GraphicsSettings);

#[derive(Resource, Default)]
pub struct ConfirmationData {
    pub message: String,
    pub has_cancel: bool,
}

#[derive(Resource)]
pub struct MenuNavigationTimer {
    pub timer: Timer,
}

impl Default for MenuNavigationTimer {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.4, TimerMode::Once);
        timer.set_elapsed(std::time::Duration::from_secs_f32(0.4));
        Self { timer }
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputDevice>()
            .init_resource::<MenuNavigationTimer>()
            .init_resource::<MenuSelectionMemory>()
            .init_resource::<PendingGraphicsSettings>()
            .init_resource::<ConfirmationData>();
        
        let menu_cond = menu_visible;

        app.add_systems(Update, device_detection_system.run_if(menu_cond.clone()))
           .add_systems(Update, menu_input_system.run_if(menu_cond.clone()))
           .add_systems(Update, menu_navigation_system.run_if(menu_cond.clone()))
           .add_systems(Update, menu_visual_system.run_if(menu_cond.clone()))
           .add_systems(Update, menu_item_system.run_if(menu_cond.clone()))
           .add_systems(Update, tooltip_system.run_if(menu_cond.clone()))
           .add_systems(Update, input_hint_system.run_if(menu_cond.clone()))
           .add_systems(Update, sync_pending_settings.run_if(in_state(GameState::Menu)));
    }
}

fn menu_visible(
    state: Res<State<GameState>>,
    exit_confirm: Res<ExitConfirmationActive>,
) -> bool {
    *state.get() == GameState::Menu || exit_confirm.0
}

fn device_detection_system(
    mut input_device: ResMut<InputDevice>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    mut gamepad_events: EventReader<GamepadEvent>,
    touches: Res<Touches>,
) {
    if keyboard.get_just_pressed().next().is_some() {
        *input_device = InputDevice::Keyboard;
    }
    if mouse_buttons.get_just_pressed().next().is_some() || mouse_motion.read().next().is_some() {
        *input_device = InputDevice::Mouse;
    }
    if gamepad_buttons.get_just_pressed().next().is_some() || gamepad_events.read().next().is_some() {
        *input_device = InputDevice::Gamepad;
    }
    if touches.any_just_pressed() || touches.any_just_released() {
        *input_device = InputDevice::Touch;
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct MenuInputParams<'w, 's> {
    pub keyboard: Res<'w, ButtonInput<KeyCode>>,
    pub gamepads: Res<'w, Gamepads>,
    pub gamepad_buttons: Res<'w, ButtonInput<GamepadButton>>,
    pub query: Query<'w, 's, &'static mut MenuContainer>,
    pub focused_query: Query<'w, 's, &'static MenuItem, With<MenuFocus>>,
    pub input_device: Res<'w, InputDevice>,
    pub next_state: ResMut<'w, NextState<GameState>>,
    pub editor_mode: ResMut<'w, EditorMode>,
    pub menu_state: Res<'w, State<MenuSubState>>,
    pub next_menu_state: ResMut<'w, NextState<MenuSubState>>,
    pub settings: ResMut<'w, GraphicsSettings>,
    pub pending: ResMut<'w, PendingGraphicsSettings>,
    pub confirmation: ResMut<'w, ConfirmationData>,
    pub time: Res<'w, Time>,
    pub nav_timer: ResMut<'w, MenuNavigationTimer>,
    pub selection_memory: ResMut<'w, MenuSelectionMemory>,
    pub game_state: Res<'w, State<GameState>>,
    pub exit_confirm: ResMut<'w, ExitConfirmationActive>,
}

pub fn menu_input_system(
    mut params: MenuInputParams,
) {
    let mut move_dir = 0;
    
    let mut up_pressed = params.keyboard.pressed(KeyCode::ArrowUp);
    let mut down_pressed = params.keyboard.pressed(KeyCode::ArrowDown);
    let mut up_just = params.keyboard.just_pressed(KeyCode::ArrowUp);
    let mut down_just = params.keyboard.just_pressed(KeyCode::ArrowDown);

    let mut select_pressed = params.keyboard.just_pressed(KeyCode::Enter) || params.keyboard.just_pressed(KeyCode::Space);
    let mut back_pressed = params.keyboard.just_pressed(KeyCode::Escape);

    for gamepad in params.gamepads.iter() {
        if params.gamepad_buttons.pressed(GamepadButton { gamepad, button_type: GamepadButtonType::DPadUp }) { up_pressed = true; }
        if params.gamepad_buttons.pressed(GamepadButton { gamepad, button_type: GamepadButtonType::DPadDown }) { down_pressed = true; }
        if params.gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::DPadUp }) { up_just = true; }
        if params.gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::DPadDown }) { down_just = true; }
        if params.gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::South }) { select_pressed = true; }
        if params.gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::East }) { back_pressed = true; }
    }

    if up_just || down_just {
        if up_just { move_dir -= 1; }
        if down_just { move_dir += 1; }
        params.nav_timer.timer.set_duration(std::time::Duration::from_secs_f32(0.4));
        params.nav_timer.timer.reset();
    } else if up_pressed || down_pressed {
        params.nav_timer.timer.tick(params.time.delta());
        if params.nav_timer.timer.finished() {
            if up_pressed { move_dir -= 1; }
            if down_pressed { move_dir += 1; }
            params.nav_timer.timer.set_duration(std::time::Duration::from_secs_f32(0.15));
            params.nav_timer.timer.reset();
        }
    } else {
        params.nav_timer.timer.set_elapsed(std::time::Duration::from_secs_f32(0.4));
    }

    if move_dir != 0 {
        for mut container in params.query.iter_mut() {
            let new_idx = (container.current_selection as i32 + move_dir).rem_euclid(container.items_count as i32);
            container.current_selection = new_idx as usize;
            params.selection_memory.selections.insert(*params.menu_state.get(), container.current_selection);
        }
    }

    if back_pressed {
        match *params.menu_state.get() {
            MenuSubState::Settings => {
                if params.pending.0 != *params.settings {
                    params.confirmation.message = "Settings have been changed, apply them?".to_string();
                    params.confirmation.has_cancel = true;
                    params.next_menu_state.set(MenuSubState::Confirmation);
                } else {
                    params.next_menu_state.set(MenuSubState::Main);
                }
            },
            MenuSubState::Confirmation => {
                params.next_menu_state.set(MenuSubState::Settings);
            },
            _ => {
                if *params.game_state.get() == GameState::Menu {
                }
            }
        }
    }

    if select_pressed {
        if let Ok(item) = params.focused_query.get_single() {
            handle_menu_action(
                item.action.clone(), 
                &mut params.next_state, 
                &mut params.editor_mode, 
                &mut params.next_menu_state, 
                &mut params.settings, 
                &mut params.pending, 
                &mut params.confirmation, 
                &params.game_state,
                &mut params.exit_confirm
            );
        }
    }

    let mut horizontal_dir = 0;
    let mut left_just = params.keyboard.just_pressed(KeyCode::ArrowLeft);
    let mut right_just = params.keyboard.just_pressed(KeyCode::ArrowRight);
    
    for gamepad in params.gamepads.iter() {
        if params.gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::DPadLeft }) { left_just = true; }
        if params.gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::DPadRight }) { right_just = true; }
    }

    if left_just { horizontal_dir = -1; }
    if right_just { horizontal_dir = 1; }

    if horizontal_dir != 0 {
        if let Ok(item) = params.focused_query.get_single() {
            if item.item_type == MenuItemType::Toggle || item.item_type == MenuItemType::Slider {
                let action = match item.action {
                    MenuAction::NextQuality | MenuAction::PrevQuality => if horizontal_dir > 0 { MenuAction::NextQuality } else { MenuAction::PrevQuality },
                    MenuAction::NextUpscaling | MenuAction::PrevUpscaling => if horizontal_dir > 0 { MenuAction::NextUpscaling } else { MenuAction::PrevUpscaling },
                    MenuAction::NextWindowMode | MenuAction::PrevWindowMode => if horizontal_dir > 0 { MenuAction::NextWindowMode } else { MenuAction::PrevWindowMode },
                    MenuAction::ToggleVSync => MenuAction::ToggleVSync,
                    _ => MenuAction::None,
                };
                if action != MenuAction::None {
                    handle_menu_action(
                        action, 
                        &mut params.next_state, 
                        &mut params.editor_mode, 
                        &mut params.next_menu_state, 
                        &mut params.settings, 
                        &mut params.pending, 
                        &mut params.confirmation, 
                        &params.game_state,
                        &mut params.exit_confirm
                    );
                }
            }
        }
    }
}

pub fn handle_menu_action(
    action: MenuAction, 
    next_game_state: &mut ResMut<NextState<GameState>>, 
    editor_mode: &mut ResMut<EditorMode>,
    next_menu_state: &mut ResMut<NextState<MenuSubState>>,
    settings: &mut ResMut<GraphicsSettings>,
    pending: &mut ResMut<PendingGraphicsSettings>,
    confirmation: &mut ResMut<ConfirmationData>,
    _game_state: &Res<State<GameState>>,
    exit_confirm: &mut ResMut<ExitConfirmationActive>,
) {
    match action {
        MenuAction::StartGame => { 
            next_game_state.set(GameState::Loading); 
            editor_mode.is_active = false; 
        },
        MenuAction::StartEditor => { 
            next_game_state.set(GameState::Loading); 
            editor_mode.is_active = true; 
        },
        MenuAction::Exit => { 
            #[cfg(not(target_arch = "wasm32"))]
            std::process::exit(0);
        },
        MenuAction::Back => { 
            if ***pending != **settings {
                confirmation.message = "Settings have been changed, apply them?".to_string();
                confirmation.has_cancel = true;
                next_menu_state.set(MenuSubState::Confirmation);
            } else {
                next_menu_state.set(MenuSubState::Main); 
            }
        },
        MenuAction::OpenSettings => { next_menu_state.set(MenuSubState::Settings); },
        MenuAction::ApplySettings => {
            if ***pending != **settings {
                **settings = (***pending).clone();
                save_settings(&**settings);
            }
        },
        MenuAction::ConfirmYes => {
            if exit_confirm.0 {
                exit_confirm.0 = false;
                editor_mode.is_active = false;
                next_game_state.set(GameState::Menu);
            } else {
                **settings = (***pending).clone();
                save_settings(&**settings);
                next_menu_state.set(MenuSubState::Main);
            }
        },
        MenuAction::ConfirmNo => {
            if exit_confirm.0 {
                exit_confirm.0 = false;
            } else {
                next_menu_state.set(MenuSubState::Main);
            }
        },
        MenuAction::ConfirmCancel => {
            if exit_confirm.0 {
                exit_confirm.0 = false;
            } else {
                next_menu_state.set(MenuSubState::Settings);
            }
        },
        MenuAction::ToggleVSync => {
            pending.vsync = !pending.vsync;
        },
        MenuAction::NextWindowMode => {
            pending.window_mode = match pending.window_mode {
                MyWindowMode::Windowed => MyWindowMode::BorderlessFullscreen,
                MyWindowMode::BorderlessFullscreen => {
                    if MyWindowMode::Fullscreen.is_supported() {
                        MyWindowMode::Fullscreen
                    } else {
                        MyWindowMode::Windowed
                    }
                },
                MyWindowMode::Fullscreen => MyWindowMode::Windowed,
            };
        },
        MenuAction::PrevWindowMode => {
            pending.window_mode = match pending.window_mode {
                MyWindowMode::Windowed => {
                    if MyWindowMode::Fullscreen.is_supported() {
                        MyWindowMode::Fullscreen
                    } else {
                        MyWindowMode::BorderlessFullscreen
                    }
                },
                MyWindowMode::BorderlessFullscreen => MyWindowMode::Windowed,
                MyWindowMode::Fullscreen => MyWindowMode::BorderlessFullscreen,
            };
        },
        MenuAction::NextUpscaling => {
            pending.upscaling = match pending.upscaling {
                UpscalingMode::None => UpscalingMode::FSR,
                UpscalingMode::FSR => UpscalingMode::TAA,
                UpscalingMode::TAA => UpscalingMode::None,
            };
        },
        MenuAction::PrevUpscaling => {
            pending.upscaling = match pending.upscaling {
                UpscalingMode::None => UpscalingMode::TAA,
                UpscalingMode::FSR => UpscalingMode::None,
                UpscalingMode::TAA => UpscalingMode::FSR,
            };
        },
        MenuAction::NextQuality => {
            pending.quality_level = match pending.quality_level {
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                QualityLevel::Ultra => QualityLevel::Low,
            };
            match pending.quality_level {
                QualityLevel::Low => { pending.shadow_quality = QualityLevel::Low; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::Medium => { pending.shadow_quality = QualityLevel::Medium; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::High => { pending.shadow_quality = QualityLevel::High; pending.fog_quality = QualityLevel::Medium; },
                QualityLevel::Ultra => { pending.shadow_quality = QualityLevel::Ultra; pending.fog_quality = QualityLevel::High; },
            }
        },
        MenuAction::PrevQuality => {
            pending.quality_level = match pending.quality_level {
                QualityLevel::Low => QualityLevel::Ultra,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                QualityLevel::Ultra => QualityLevel::High,
            };
            match pending.quality_level {
                QualityLevel::Low => { pending.shadow_quality = QualityLevel::Low; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::Medium => { pending.shadow_quality = QualityLevel::Medium; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::High => { pending.shadow_quality = QualityLevel::High; pending.fog_quality = QualityLevel::Medium; },
                QualityLevel::Ultra => { pending.shadow_quality = QualityLevel::Ultra; pending.fog_quality = QualityLevel::High; },
            }
        },
        _ => {}
    }
}

fn menu_navigation_system(
    mut commands: Commands,
    container_query: Query<&MenuContainer, Changed<MenuContainer>>,
    item_query: Query<(Entity, &MenuItem)>,
    menu_state: Res<State<MenuSubState>>,
    settings: Res<GraphicsSettings>,
) {
    if menu_state.is_changed() || settings.is_changed() { return; }

    for container in container_query.iter() {
        for (entity, item) in item_query.iter() {
            if let Some(mut e) = commands.get_entity(entity) {
                if item.index == container.current_selection {
                    e.insert(MenuFocus);
                } else {
                    e.remove::<MenuFocus>();
                }
            }
        }
    }
}

fn tooltip_system(
    _commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MenuTooltip, &mut Visibility)>,
) {
    for (_entity, mut tooltip, mut visibility) in query.iter_mut() {
        tooltip.timer.tick(time.delta());
        if tooltip.timer.finished() {
            *visibility = Visibility::Hidden;
        }
    }
}

fn input_hint_system(
    input_device: Res<InputDevice>,
    mut query: Query<&mut Text, With<InputHintFooter>>,
) {
    for mut text in query.iter_mut() {
        let hint = match *input_device {
            InputDevice::Keyboard => "[Arrows] Navigate/Change  [Enter] Select  [Esc] Back",
            InputDevice::Gamepad => "(D-Pad) Navigate/Change  (A) Select  (B) Back",
            InputDevice::Mouse => "Hover to Focus  Click to Cycle/Select",
            InputDevice::Touch => "Tap to Select  Long Press for Hint",
        };
        text.sections[0].value = hint.to_string();
    }
}

fn menu_visual_system(
    mut query: Query<(&MenuItem, &mut BackgroundColor, &mut Transform, Option<&MenuFocus>)>,
    settings: Res<GraphicsSettings>,
    pending: Res<PendingGraphicsSettings>,
) {
    let has_changes = **pending != *settings;
    for (item, mut color, mut transform, focus) in query.iter_mut() {
        let is_apply = item.action == MenuAction::ApplySettings;
        if focus.is_some() {
            if is_apply && !has_changes {
                *color = Color::srgb(0.2, 0.2, 0.2).into();
            } else {
                *color = Color::srgb(0.4, 0.4, 0.5).into();
            }
            transform.scale = Vec3::splat(1.05);
        } else {
            if is_apply && !has_changes {
                *color = Color::srgb(0.1, 0.1, 0.1).into();
            } else {
                *color = Color::srgb(0.2, 0.2, 0.2).into();
            }
            transform.scale = Vec3::splat(1.0);
        }
    }
}

pub fn spawn_menu_button(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    text: &str,
    index: usize,
    item_type: MenuItemType,
    action: MenuAction,
    tooltip: Option<String>,
) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(300.0), height: Val::Px(60.0),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(20.0)),
                ..default()
            },
            border_color: Color::WHITE.into(),
            background_color: Color::srgb(0.2, 0.2, 0.2).into(),
            ..default()
        },
        MenuItem { index, item_type, action, tooltip },
    )).with_children(|p| {
        p.spawn(TextBundle::from_section(
            text,
            TextStyle { font: font.clone(), font_size: 24.0, color: Color::WHITE },
        ));

        if item_type == MenuItemType::Submenu {
            p.spawn(TextBundle::from_section(
                "→",
                TextStyle { font: font.clone(), font_size: 24.0, color: Color::WHITE },
            ));
        }
    });
}

#[derive(Component)]
pub struct MenuItemRoot;

pub fn setup_menu(
    mut commands: Commands, 
    _asset_server: Res<AssetServer>, 
    game_state: Res<State<GameState>>,
    exit_confirm: Res<ExitConfirmationActive>,
    camera_query: Query<Entity, With<Camera2d>>,
    mut next_menu_state: ResMut<NextState<MenuSubState>>,
) {
    use crate::MenuEntity;
    
    // If we are in the main menu state and NOT in exit confirmation, ensure we are in Main substate
    if *game_state.get() == GameState::Menu {
        next_menu_state.set(MenuSubState::Main);
    }
    
    // Check if a 2D camera already exists
    if camera_query.is_empty() {
        commands.spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 10,
                    ..default()
                },
                ..default()
            },
            MenuEntity,
        ));
    }
    
    let bg_color = if exit_confirm.0 && *game_state.get() == GameState::InGame {
        Color::srgba(0.01, 0.01, 0.05, 0.8)
    } else {
        Color::srgba(0.01, 0.01, 0.05, 1.0)
    };

    // Root Container for items
    commands.spawn((NodeBundle {
        style: Style {
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            display: Display::Flex, flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            row_gap: Val::Px(20.0),
            ..default()
        },
        background_color: bg_color.into(),
        ..default()
    }, MenuEntity, MenuItemRoot));

    // Footer for input hints
    commands.spawn((NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        z_index: ZIndex::Global(100),
        ..default()
    }, MenuEntity)).with_children(|p| {
        p.spawn((TextBundle::from_section(
            "",
            TextStyle { font_size: 18.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() },
        ), InputHintFooter));
    });
}

fn menu_item_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu_state: Res<State<MenuSubState>>,
    settings: Res<GraphicsSettings>,
    pending: Res<PendingGraphicsSettings>,
    confirmation: Res<ConfirmationData>,
    extra_buttons: Res<ExtraMenuButtons>,
    root_query: Query<(Entity, Option<&Children>), With<MenuItemRoot>>,
    selection_memory: Res<MenuSelectionMemory>,
) {
    let Ok((root, children)) = root_query.get_single() else { return; };
    let is_empty = children.map_or(true, |c| c.is_empty());
    
    if !menu_state.is_changed() && !settings.is_changed() && !pending.is_changed() && !is_empty { return; }
    
    // Despawn old items (all children of root)
    if let Some(children) = children {
        for &child in children.iter() {
            commands.entity(child).despawn_recursive();
        }
    }
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let mut new_container = None;

    commands.entity(root).with_children(|parent| {
        match menu_state.get() {
            MenuSubState::Main => {
                parent.spawn(TextBundle::from_section(
                    "Klep2tron",
                    TextStyle { font: font.clone(), font_size: 100.0, color: Color::WHITE },
                ).with_style(Style { margin: UiRect::bottom(Val::Px(40.0)), ..default() }));

                spawn_menu_button(parent, &font, "START GAME", 0, MenuItemType::Action, MenuAction::StartGame, Some("Start a new game session".to_string()));
                
                for (idx, (label, action)) in extra_buttons.buttons.iter().enumerate() {
                    spawn_menu_button(parent, &font, label, idx + 1, MenuItemType::Action, action.clone(), None);
                }

                spawn_menu_button(parent, &font, "SETTINGS", extra_buttons.buttons.len() + 1, MenuItemType::Submenu, MenuAction::OpenSettings, Some("Graphics and performance".to_string()));
                
                #[cfg(not(target_arch = "wasm32"))]
                spawn_menu_button(parent, &font, "EXIT", extra_buttons.buttons.len() + 2, MenuItemType::Action, MenuAction::Exit, None);
                
                // Update items count
                #[cfg(not(target_arch = "wasm32"))]
                let count = 3 + extra_buttons.buttons.len();
                #[cfg(target_arch = "wasm32")]
                let count = 2 + extra_buttons.buttons.len();
                
                new_container = Some(MenuContainer { current_selection: 0, items_count: count });
            },
            MenuSubState::Settings => {
                parent.spawn(TextBundle::from_section(
                    "Settings",
                    TextStyle { font: font.clone(), font_size: 60.0, color: Color::WHITE },
                ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default() }));

                spawn_menu_button(parent, &font, "BACK", 0, MenuItemType::Action, MenuAction::Back, None);
                
                let quality_text = format!("QUALITY: {:?}", pending.quality_level);
                spawn_menu_button(parent, &font, &quality_text, 1, MenuItemType::Toggle, MenuAction::NextQuality, Some("Global quality preset".to_string()));

                let upscale_text = format!("UPSCALING: {:?}", pending.upscaling);
                spawn_menu_button(parent, &font, &upscale_text, 2, MenuItemType::Toggle, MenuAction::NextUpscaling, Some("FSR 1.0 or TAA".to_string()));

                let vsync_text = format!("VSYNC: {}", if pending.vsync { "ON" } else { "OFF" });
                spawn_menu_button(parent, &font, &vsync_text, 3, MenuItemType::Toggle, MenuAction::ToggleVSync, None);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let mode_text = format!("MODE: {:?}", pending.window_mode);
                    spawn_menu_button(parent, &font, &mode_text, 4, MenuItemType::Toggle, MenuAction::NextWindowMode, None);
                    
                    let _has_changes = *settings != **pending;
                    // We can't easily change color per button in spawn_menu_button without extra param, but we'll use a hack or just pass it.
                    spawn_menu_button(parent, &font, "APPLY", 5, MenuItemType::Action, MenuAction::ApplySettings, None);
                }
                
                let count = if cfg!(target_arch = "wasm32") { 4 } else { 6 };
                new_container = Some(MenuContainer { current_selection: 0, items_count: count });
            },
            MenuSubState::Confirmation => {
                parent.spawn(TextBundle::from_section(
                    &confirmation.message,
                    TextStyle { font: font.clone(), font_size: 40.0, color: Color::WHITE },
                ).with_style(Style { margin: UiRect::bottom(Val::Px(40.0)), ..default() }));

                spawn_menu_button(parent, &font, "YES", 0, MenuItemType::Action, MenuAction::ConfirmYes, None);
                spawn_menu_button(parent, &font, "NO", 1, MenuItemType::Action, MenuAction::ConfirmNo, None);
                
                let mut count = 2;
                if confirmation.has_cancel {
                    spawn_menu_button(parent, &font, "CANCEL", 2, MenuItemType::Action, MenuAction::ConfirmCancel, None);
                    count = 3;
                }
                
                new_container = Some(MenuContainer { current_selection: 0, items_count: count });
            }
        }
    });

    if let Some(container) = new_container {
        let mut final_container = container;
        // Restore selection from memory if available
        if let Some(&saved_idx) = selection_memory.selections.get(menu_state.get()) {
            // Ensure we don't restore an out-of-bounds index (e.g. if items count changed)
            if saved_idx < final_container.items_count {
                final_container.current_selection = saved_idx;
            }
        }
        commands.entity(root).insert(final_container);
    }
}

fn sync_pending_settings(
    menu_state: Res<State<MenuSubState>>,
    settings: Res<GraphicsSettings>,
    mut pending: ResMut<PendingGraphicsSettings>,
) {
    if menu_state.is_changed() && *menu_state.get() == MenuSubState::Settings {
        **pending = (*settings).clone();
    }
}
