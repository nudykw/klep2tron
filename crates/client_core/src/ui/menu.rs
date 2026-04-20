use bevy::prelude::*;
use bevy::input::gamepad::GamepadEvent;
use crate::GameState;

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
    SetResolution(u32, u32),
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

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputDevice>()
            .add_systems(Update, (
                device_detection_system,
                menu_input_system,
                menu_navigation_system,
                menu_visual_system,
                tooltip_system,
                input_hint_system,
            ));
    }
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

fn menu_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    mut query: Query<&mut MenuContainer>,
    focused_query: Query<&MenuItem, With<MenuFocus>>,
    _input_device: Res<InputDevice>,
    mut next_state: ResMut<NextState<GameState>>,
    mut editor_mode: ResMut<crate::EditorMode>,
) {
    let mut move_dir = 0;
    
    if keyboard.just_pressed(KeyCode::ArrowUp) { move_dir -= 1; }
    if keyboard.just_pressed(KeyCode::ArrowDown) { move_dir += 1; }

    let mut select_pressed = keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space);
    let mut back_pressed = keyboard.just_pressed(KeyCode::Escape);

    for gamepad in gamepads.iter() {
        if gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::DPadUp }) { move_dir -= 1; }
        if gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::DPadDown }) { move_dir += 1; }
        if gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::South }) { select_pressed = true; }
        if gamepad_buttons.just_pressed(GamepadButton { gamepad, button_type: GamepadButtonType::East }) { back_pressed = true; }
    }

    if move_dir != 0 {
        for mut container in query.iter_mut() {
            let new_idx = (container.current_selection as i32 + move_dir).rem_euclid(container.items_count as i32);
            container.current_selection = new_idx as usize;
        }
    }

    if select_pressed {
        if let Ok(item) = focused_query.get_single() {
            handle_menu_action(item.action.clone(), &mut next_state, &mut editor_mode);
        }
    }

    if back_pressed {
        // For now, Back always returns to Menu if we are in Loading or InGame
        next_state.set(GameState::Menu);
    }
}

fn handle_menu_action(
    action: MenuAction, 
    next_state: &mut ResMut<NextState<GameState>>, 
    editor_mode: &mut ResMut<crate::EditorMode>
) {
    match action {
        MenuAction::StartGame => {
            editor_mode.is_active = false;
            next_state.set(GameState::Loading);
        }
        MenuAction::StartEditor => {
            editor_mode.is_active = true;
            next_state.set(GameState::Loading);
        }
        MenuAction::Exit => {
            #[cfg(not(target_arch = "wasm32"))]
            std::process::exit(0);
        }
        MenuAction::Back => {
            next_state.set(GameState::Menu);
        }
        _ => {}
    }
}

fn menu_navigation_system(
    mut commands: Commands,
    container_query: Query<&MenuContainer, Changed<MenuContainer>>,
    item_query: Query<(Entity, &MenuItem)>,
    _tooltip_query: Query<(Entity, &mut MenuTooltip, &mut Style)>,
) {
    for container in container_query.iter() {
        for (entity, item) in item_query.iter() {
            if item.index == container.current_selection {
                commands.entity(entity).insert(MenuFocus);
                // Trigger tooltip if available
                if let Some(_text) = &item.tooltip {
                    // Spawn or update tooltip logic here
                }
            } else {
                commands.entity(entity).remove::<MenuFocus>();
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
    if !input_device.is_changed() { return; }
    for mut text in query.iter_mut() {
        let hint = match *input_device {
            InputDevice::Keyboard => "[Arrows] Navigate  [Enter] Select  [Esc] Back",
            InputDevice::Gamepad => "(D-Pad) Navigate  (A) Select  (B) Back",
            InputDevice::Mouse => "Hover to Focus  Click to Select",
            InputDevice::Touch => "Tap to Select  Long Press for Hint",
        };
        text.sections[0].value = hint.to_string();
    }
}

fn menu_visual_system(
    mut query: Query<(&mut BackgroundColor, &mut Transform, Option<&MenuFocus>), With<MenuItem>>,
) {
    for (mut color, mut transform, focus) in query.iter_mut() {
        if focus.is_some() {
            *color = Color::srgb(0.4, 0.4, 0.5).into();
            transform.scale = Vec3::splat(1.05);
        } else {
            *color = Color::srgb(0.2, 0.2, 0.2).into();
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
