use bevy::prelude::*;

pub fn fullscreen_toggle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && keyboard.just_pressed(KeyCode::Enter) {
        if let Ok(mut window) = windows.get_single_mut() {
            use bevy::window::WindowMode;
            window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::SizedFullscreen,
                _ => WindowMode::Windowed,
            };
        }
    }
}
