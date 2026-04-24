use bevy::prelude::*;
use bevy::input::gamepad::GamepadEvent;
use crate::{GameState, EditorMode};
use super::super::types::*;
use super::actions::handle_menu_action;

#[derive(bevy::ecs::system::SystemParam)]
pub struct MenuInputParams<'w, 's> {
    pub keyboard: Res<'w, ButtonInput<KeyCode>>,
    pub gamepads: Res<'w, Gamepads>,
    pub gamepad_buttons: Res<'w, ButtonInput<GamepadButton>>,
    pub query: Query<'w, 's, (Entity, &'static mut MenuContainer)>,
    pub overlay_query: Query<'w, 's, Entity, With<ConfirmationOverlay>>,
    pub focused_query_with_entity: Query<'w, 's, (Entity, &'static MenuItem), With<MenuFocus>>,
    pub focused_query: Query<'w, 's, &'static MenuItem, With<MenuFocus>>,
    pub input_device: Res<'w, InputDevice>,
    pub next_state: ResMut<'w, NextState<GameState>>,
    pub editor_mode: ResMut<'w, EditorMode>,
    pub menu_state: Res<'w, State<MenuSubState>>,
    pub next_menu_state: ResMut<'w, NextState<MenuSubState>>,
    pub settings: ResMut<'w, crate::GraphicsSettings>,
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
    pub parent_query: Query<'w, 's, &'static Parent>,
}

pub fn device_detection_system(
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

pub fn menu_input_system(
    mut params: MenuInputParams,
) {
    let is_confirmation = *params.menu_state.get() == MenuSubState::Confirmation;
    let has_overlay = !params.overlay_query.is_empty();
    
    if is_confirmation && !has_overlay {
        return;
    }

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
        if has_overlay {
            for (entity, mut container) in params.query.iter_mut() {
                if params.overlay_query.get(entity).is_ok() {
                    update_container_logic(&mut container, move_dir, &params.item_query);
                }
            }
        } else if !is_confirmation {
            for (_entity, mut container) in params.query.iter_mut() {
                update_container_logic(&mut container, move_dir, &params.item_query);
                params.selection_memory.selections.insert(*params.menu_state.get(), container.current_selection);
            }
        }
    }

    if back_pressed {
        let action = if is_confirmation { MenuAction::ConfirmCancel } else { MenuAction::Back };
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

    if select_pressed {
        let has_overlay = !params.overlay_query.is_empty();
        let mut target_item = None;
        for (item_entity, item) in params.focused_query_with_entity.iter() {
            let is_in_overlay = is_child_of_any(item_entity, &params.overlay_query, &params.parent_query);
            if has_overlay && !is_in_overlay { continue; }
            target_item = Some(item.action.clone());
            break;
        }

        if let Some(action) = target_item {
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
        if !is_confirmation {
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
}

fn is_child_of_any(
    entity: Entity,
    targets: &Query<Entity, With<ConfirmationOverlay>>,
    parent_query: &Query<&Parent>,
) -> bool {
    let mut current = entity;
    loop {
        if targets.get(current).is_ok() {
            return true;
        }
        if let Ok(parent) = parent_query.get(current) {
            current = parent.get();
        } else {
            break;
        }
    }
    false
}

fn update_container_logic(
    container: &mut MenuContainer,
    move_dir: i32,
    item_query: &Query<&MenuItem>,
) {
    let mut next_idx = container.current_selection;
    let start_idx = next_idx;
    loop {
        next_idx = (next_idx as i32 + move_dir).rem_euclid(container.items_count as i32) as usize;
        
        let mut is_disabled = false;
        for item in item_query.iter() {
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
}
