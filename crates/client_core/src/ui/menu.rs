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
    OpenAdvanced,
    NextGpu, PrevGpu,
    NextSsao, PrevSsao,
    ToggleBloom,
    NextFog, PrevFog,
    NextShadowRes, PrevShadowRes,
    NextShadowQuality, PrevShadowQuality,
    ToggleFpsLimit,
    NextFpsLimit, PrevFpsLimit,
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
    Advanced,
}

#[derive(Component)]
pub struct MenuContainer {
    pub current_selection: usize,
    pub items_count: usize,
}

#[derive(Component)]
pub struct MenuScrollContainer;

#[derive(Component)]
pub struct MenuViewport;

#[derive(Component)]
pub struct MenuTitleDisplay;

#[derive(Component)]
pub struct MenuItem {
    pub index: usize,
    pub item_type: MenuItemType,
    pub action: MenuAction,
    pub tooltip: Option<String>,
    pub is_disabled: bool,
}

#[derive(Component)]
pub struct MenuFocus; 

#[derive(Component)]
pub struct MenuTooltip {
    pub timer: Timer,
}

#[derive(Component)]
pub struct InputHintFooter;

#[derive(Component)]
pub struct TooltipDisplay;

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

        app.add_systems(Update, (
                device_detection_system,
                menu_input_system,
                menu_item_system,
                apply_deferred,
                menu_navigation_system,
                menu_scrolling_system,
                menu_visual_system,
                tooltip_system,
                input_hint_system,
                menu_tooltip_system,
           ).chain().run_if(menu_cond.clone()))

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
    pub item_query: Query<'w, 's, &'static MenuItem>,
    pub gpu_list: ResMut<'w, crate::settings::GpuList>,
    pub instance_adapter: Option<Res<'w, bevy::render::renderer::RenderAdapterInfo>>,
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
            let mut next_idx = container.current_selection;
            let start_idx = next_idx;
            loop {
                next_idx = (next_idx as i32 + move_dir).rem_euclid(container.items_count as i32) as usize;
                
                let mut is_disabled = false;
                for item in params.item_query.iter() {
                    if item.index == next_idx && item.is_disabled {
                        is_disabled = true;
                        break;
                    }
                }
                
                if !is_disabled || next_idx == start_idx {
                    break;
                }
            }
            container.current_selection = next_idx;
            params.selection_memory.selections.insert(*params.menu_state.get(), container.current_selection);
        }
    }

    if back_pressed {

        handle_menu_action(
            MenuAction::Back, 
            &mut params.next_state, 
            &mut params.editor_mode, 
            &mut params.next_menu_state, 
            &mut params.settings, 
            &mut params.pending, 
            &mut params.confirmation, 
            &params.game_state,
            &mut params.exit_confirm,
            &mut params.gpu_list,
            &params.instance_adapter,
            &params.menu_state,
        );
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
                &mut params.exit_confirm,
                &mut params.gpu_list,
                &params.instance_adapter,
                &params.menu_state,
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
                    MenuAction::NextGpu | MenuAction::PrevGpu => if horizontal_dir > 0 { MenuAction::NextGpu } else { MenuAction::PrevGpu },
                    MenuAction::NextSsao | MenuAction::PrevSsao => if horizontal_dir > 0 { MenuAction::NextSsao } else { MenuAction::PrevSsao },
                    MenuAction::NextFog | MenuAction::PrevFog => if horizontal_dir > 0 { MenuAction::NextFog } else { MenuAction::PrevFog },
                    MenuAction::NextShadowRes | MenuAction::PrevShadowRes => if horizontal_dir > 0 { MenuAction::NextShadowRes } else { MenuAction::PrevShadowRes },
                    MenuAction::NextShadowQuality | MenuAction::PrevShadowQuality => if horizontal_dir > 0 { MenuAction::NextShadowQuality } else { MenuAction::PrevShadowQuality },
                    MenuAction::NextFpsLimit | MenuAction::PrevFpsLimit => if horizontal_dir > 0 { MenuAction::NextFpsLimit } else { MenuAction::PrevFpsLimit },
                    MenuAction::ToggleVSync => MenuAction::ToggleVSync,
                    MenuAction::ToggleBloom => MenuAction::ToggleBloom,
                    MenuAction::ToggleFpsLimit => MenuAction::ToggleFpsLimit,
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
                        &mut params.exit_confirm,
                        &mut params.gpu_list,
                        &params.instance_adapter,
                        &params.menu_state,
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
    gpu_list: &mut ResMut<crate::settings::GpuList>,
    instance_adapter: &Option<Res<bevy::render::renderer::RenderAdapterInfo>>,
    menu_state: &Res<State<MenuSubState>>,
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
            let has_changes = ***pending != **settings;
            match *menu_state.get() {
                MenuSubState::Advanced => {
                    if has_changes {
                        if pending.selected_gpu != settings.selected_gpu {
                            confirmation.message = "GPU changed. Save and restart app?".to_string();
                        } else {
                            confirmation.message = "Advanced settings changed, apply them?".to_string();
                        }
                        confirmation.has_cancel = true;
                        next_menu_state.set(MenuSubState::Confirmation);
                    } else {
                        next_menu_state.set(MenuSubState::Settings);
                    }
                },
                MenuSubState::Settings => {
                    if has_changes {
                        if pending.selected_gpu != settings.selected_gpu {
                            confirmation.message = "GPU changed. Save and restart app?".to_string();
                        } else {
                            confirmation.message = "Settings have been changed, apply them?".to_string();
                        }
                        confirmation.has_cancel = true;
                        next_menu_state.set(MenuSubState::Confirmation);
                    } else {
                        next_menu_state.set(MenuSubState::Main); 
                    }
                },
                _ => {
                    next_menu_state.set(MenuSubState::Main);
                }
            }
        },
        MenuAction::OpenSettings => { next_menu_state.set(MenuSubState::Settings); },
        MenuAction::OpenAdvanced => { 
            next_menu_state.set(MenuSubState::Advanced); 
            // Refresh GPU list when entering advanced
            crate::settings::populate_gpu_list(gpu_list, instance_adapter.as_ref().map(|v| &**v));
        },
        MenuAction::NextSsao => {
            pending.ssao = match pending.ssao {
                QualityLevel::Off => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                _ => QualityLevel::Off,
            };
        },
        MenuAction::PrevSsao => {
            pending.ssao = match pending.ssao {
                QualityLevel::Off => QualityLevel::Ultra,
                QualityLevel::Low => QualityLevel::Off,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                _ => QualityLevel::High,
            };
        },
        MenuAction::ToggleBloom => {
            pending.bloom = !pending.bloom;
        },
        MenuAction::NextFog => {
            pending.fog_quality = match pending.fog_quality {
                QualityLevel::Off => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                _ => QualityLevel::Off,
            };
        },
        MenuAction::PrevFog => {
            pending.fog_quality = match pending.fog_quality {
                QualityLevel::Off => QualityLevel::Ultra,
                QualityLevel::Low => QualityLevel::Off,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                _ => QualityLevel::High,
            };
        },
        MenuAction::NextShadowRes => {
            pending.shadow_resolution = match pending.shadow_resolution {
                512 => 1024,
                1024 => 2048,
                2048 => 4096,
                _ => 512,
            };
        },
        MenuAction::PrevShadowRes => {
            pending.shadow_resolution = match pending.shadow_resolution {
                512 => 4096,
                1024 => 512,
                2048 => 1024,
                _ => 2048,
            };
        },
        MenuAction::NextShadowQuality => {
            pending.shadow_quality = match pending.shadow_quality {
                QualityLevel::Off => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                _ => QualityLevel::Off,
            };
        },
        MenuAction::PrevShadowQuality => {
            pending.shadow_quality = match pending.shadow_quality {
                QualityLevel::Off => QualityLevel::Ultra,
                QualityLevel::Low => QualityLevel::Off,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                _ => QualityLevel::High,
            };
        },
        MenuAction::ToggleFpsLimit => {
            pending.fps_limit_enabled = !pending.fps_limit_enabled;
        },
        MenuAction::NextFpsLimit => {
            pending.fps_limit = match pending.fps_limit {
                30 => 60,
                60 => 120,
                120 => 144,
                144 => 240,
                _ => 30,
            };
        },
        MenuAction::PrevFpsLimit => {
            pending.fps_limit = match pending.fps_limit {
                30 => 240,
                60 => 30,
                120 => 60,
                144 => 120,
                _ => 144,
            };
        },
        MenuAction::NextGpu => {
            if !gpu_list.names.is_empty() {
                let current = settings.selected_gpu.clone().unwrap_or_else(|| gpu_list.names[0].clone());
                if let Some(idx) = gpu_list.names.iter().position(|n| n == &current) {
                    let next_idx = (idx + 1) % gpu_list.names.len();
                    pending.selected_gpu = Some(gpu_list.names[next_idx].clone());
                }
            }
        },
        MenuAction::PrevGpu => {
            if !gpu_list.names.is_empty() {
                let current = settings.selected_gpu.clone().unwrap_or_else(|| gpu_list.names[0].clone());
                if let Some(idx) = gpu_list.names.iter().position(|n| n == &current) {
                    let next_idx = (idx + gpu_list.names.len() - 1) % gpu_list.names.len();
                    pending.selected_gpu = Some(gpu_list.names[next_idx].clone());
                }
            }
        },
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
                let gpu_changed = pending.selected_gpu != settings.selected_gpu;
                **settings = (***pending).clone();
                save_settings(&**settings);
                
                if gpu_changed {
                    #[cfg(not(target_arch = "wasm32"))]
                    std::process::exit(0);
                }
                
                next_menu_state.set(MenuSubState::Main);
            }
        },
        MenuAction::ConfirmNo => {
            if exit_confirm.0 {
                exit_confirm.0 = false;
            } else {
                next_menu_state.set(MenuSubState::Settings);
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
                _ => QualityLevel::Medium,
            };
            match pending.quality_level {
                QualityLevel::Low => { pending.shadow_quality = QualityLevel::Low; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::Medium => { pending.shadow_quality = QualityLevel::Medium; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::High => { pending.shadow_quality = QualityLevel::High; pending.fog_quality = QualityLevel::Medium; },
                QualityLevel::Ultra => { pending.shadow_quality = QualityLevel::Ultra; pending.fog_quality = QualityLevel::High; },
                _ => {},
            }
        },
        MenuAction::PrevQuality => {
            pending.quality_level = match pending.quality_level {
                QualityLevel::Low => QualityLevel::Ultra,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                QualityLevel::Ultra => QualityLevel::High,
                _ => QualityLevel::Medium,
            };
            match pending.quality_level {
                QualityLevel::Low => { pending.shadow_quality = QualityLevel::Low; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::Medium => { pending.shadow_quality = QualityLevel::Medium; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::High => { pending.shadow_quality = QualityLevel::High; pending.fog_quality = QualityLevel::Medium; },
                QualityLevel::Ultra => { pending.shadow_quality = QualityLevel::Ultra; pending.fog_quality = QualityLevel::High; },
                _ => {},
            }
        },
        _ => {}
    }
}

fn menu_navigation_system(
    mut commands: Commands,
    container_query: Query<&MenuContainer, Changed<MenuContainer>>,
    item_query: Query<(Entity, &MenuItem)>,
    _menu_state: Res<State<MenuSubState>>,
    _settings: Res<GraphicsSettings>,
) {
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

fn menu_scrolling_system(
    container_query: Query<&MenuContainer, With<MenuItemRoot>>,
    mut scroll_query: Query<&mut Style, With<MenuScrollContainer>>,
) {
    let Ok(container) = container_query.get_single() else { return; };
    for mut style in scroll_query.iter_mut() {
        let item_height = 54.0; 
        let viewport_height = 400.0;
        // Center the selected item in the viewport
        let target_offset = (viewport_height / 2.0) - (container.current_selection as f32 * item_height) - (item_height / 2.0);
        
        style.top = Val::Px(target_offset);
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

fn menu_tooltip_system(
    focus_query: Query<&MenuItem, With<MenuFocus>>,
    mut tooltip_query: Query<&mut Text, With<TooltipDisplay>>,
) {
    let mut text_val = "".to_string();
    if let Ok(item) = focus_query.get_single() {
        if let Some(tooltip) = &item.tooltip {
            text_val = tooltip.clone();
        }
    }
    
    for mut text in tooltip_query.iter_mut() {
        if text.sections[0].value != text_val {
            text.sections[0].value = text_val.clone();
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
    time: Res<Time>,
    mut query: Query<(&MenuItem, &mut BackgroundColor, &mut BorderColor, &mut Transform, Option<&MenuFocus>)>,
    settings: Res<GraphicsSettings>,
    pending: Res<PendingGraphicsSettings>,
) {
    let has_changes = **pending != *settings;
    let t = (time.elapsed_seconds() * 3.0).sin() * 0.5 + 0.5;

    for (item, mut bg, mut border, mut transform, focus) in query.iter_mut() {
        let is_apply = item.action == MenuAction::ApplySettings;
        let is_dimmed = item.is_disabled || (is_apply && !has_changes);

        if is_dimmed {
            *bg = Color::srgba(0.1, 0.1, 0.1, 0.2).into();
            *border = Color::NONE.into();
            transform.scale = Vec3::splat(1.0);
        } else if focus.is_some() {
            // Pulse effect for focused item
            let bg_alpha = 0.15 + t * 0.15;
            let border_alpha = 0.4 + t * 0.4;
            *bg = Color::srgba(0.3, 0.6, 1.0, bg_alpha).into();
            *border = Color::srgba(0.4, 0.7, 1.0, border_alpha).into();
            transform.scale = Vec3::splat(1.05);
        } else {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
            *border = Color::NONE.into();
            transform.scale = Vec3::splat(1.0);
        }
    }
}


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

#[derive(Component)]
pub struct MenuItemRoot;

pub fn setup_menu(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
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
        // Spawn 3D camera for background (skybox)
        commands.spawn((
            Camera3dBundle {
                camera: Camera {
                    order: 0,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(1.0, 0.0, 0.0), Vec3::Y),
                ..default()
            },
            MenuEntity,
        ));

        // Spawn 2D camera for UI
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

    // Root Container for items
    commands.spawn((NodeBundle {
        style: Style {
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            display: Display::Flex, flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center, justify_content: JustifyContent::FlexStart,
            position_type: PositionType::Absolute,
            row_gap: Val::Px(0.0),
            padding: UiRect::top(Val::Px(60.0)),
            ..default()
        },
        background_color: if exit_confirm.0 { Color::srgba(0.0, 0.0, 0.05, 0.6).into() } else { Color::NONE.into() },
        ..default()
    }, MenuEntity, MenuItemRoot)).with_children(|p| {
        // Persistent Title
        p.spawn((TextBundle::from_section(
            "Klep2tron",
            TextStyle { font: font.clone(), font_size: 70.0, color: Color::WHITE },
        ).with_style(Style { 
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        }), MenuTitleDisplay));

        // Viewport to clip scrolling items (with glass effect)
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

    // Footer for input hints and tooltips
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

fn menu_item_system(
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
    
    // Update title text
    if let Ok(mut text) = title_query.get_single_mut() {
        text.sections[0].value = match *menu_state.get() {
            MenuSubState::Main => "Klep2tron".to_string(),
            MenuSubState::Settings => "Settings".to_string(),
            MenuSubState::Confirmation => "Confirmation".to_string(),
            MenuSubState::Advanced => "Advanced".to_string(),
        };
    }
    
    // Clear viewport children (the scroll container)
    if let Some(v) = viewport_entity {
        commands.entity(v).despawn_descendants();
    }

    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let mut new_container = None;

    // Spawn items in viewport
    if let Some(v) = viewport_entity {
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
                    MenuSubState::Confirmation => {
                        scroll_p.spawn(TextBundle::from_section(
                            &confirmation.message,
                            TextStyle { font: font.clone(), font_size: 40.0, color: Color::WHITE },
                        ).with_style(Style { margin: UiRect::bottom(Val::Px(40.0)), ..default() }));

                        spawn_menu_button(scroll_p, &font, "YES", None, 0, MenuItemType::Action, MenuAction::ConfirmYes, None, false);
                        spawn_menu_button(scroll_p, &font, "NO", None, 1, MenuItemType::Action, MenuAction::ConfirmNo, None, false);
                        
                        let mut count = 2;
                        if confirmation.has_cancel {
                            spawn_menu_button(scroll_p, &font, "CANCEL", None, 2, MenuItemType::Action, MenuAction::ConfirmCancel, None, false);
                            count = 3;
                        }
                        new_container = Some(MenuContainer { current_selection: 0, items_count: count });
                    },
                    MenuSubState::Advanced => {
                        spawn_menu_button(scroll_p, &font, "BACK", None, 0, MenuItemType::Action, MenuAction::Back, None, false);
                        
                        let gpu_val = pending.selected_gpu.clone().unwrap_or_else(|| {
                            if gpu_list.names.is_empty() { "Detecting...".to_string() } else { gpu_list.names[0].clone() }
                        });
                        spawn_menu_button(scroll_p, &font, "GPU", Some(gpu_val), 1, MenuItemType::Toggle, MenuAction::NextGpu, Some("Select graphics hardware".to_string()), false);
                        spawn_menu_button(scroll_p, &font, "SHADOWS", Some(format!("{:?}", pending.shadow_quality)), 2, MenuItemType::Toggle, MenuAction::NextShadowQuality, None, false);
                        spawn_menu_button(scroll_p, &font, "FOG", Some(format!("{:?}", pending.fog_quality)), 3, MenuItemType::Toggle, MenuAction::NextFog, None, false);
                        spawn_menu_button(scroll_p, &font, "BLOOM", Some(if pending.bloom { "ON" } else { "OFF" }.to_string()), 4, MenuItemType::Toggle, MenuAction::ToggleBloom, None, false);
                        spawn_menu_button(scroll_p, &font, "SSAO", Some(format!("{:?}", pending.ssao)), 5, MenuItemType::Toggle, MenuAction::NextSsao, None, false);
                        spawn_menu_button(scroll_p, &font, "SHADOW RES", Some(pending.shadow_resolution.to_string()), 6, MenuItemType::Toggle, MenuAction::NextShadowRes, None, false);
                        spawn_menu_button(scroll_p, &font, "FPS LIMIT", Some(if pending.fps_limit_enabled { "ON" } else { "OFF" }.to_string()), 7, MenuItemType::Toggle, MenuAction::ToggleFpsLimit, None, false);
                        spawn_menu_button(scroll_p, &font, "FPS VALUE", Some(pending.fps_limit.to_string()), 8, MenuItemType::Toggle, MenuAction::NextFpsLimit, None, !pending.fps_limit_enabled);

                        new_container = Some(MenuContainer { current_selection: 0, items_count: 9 });
                    }
                }
            });
        });
    }

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
