//! Gamepad injection (`POST /gamepad`) and a small read-only helper.
//!
//! A virtual [`Gamepad`] entity is spawned on first use; input is injected as
//! the same [`RawGamepadEvent`]s `bevy_gilrs` emits. Bevy clears the digital
//! button state every frame and re-applies these events, so one injected press
//! yields `just_pressed` for exactly one frame (a "tap").

use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
use bevy::prelude::*;

/// Name of the virtual pad, visible in `GET /scene_tree`.
pub(super) const VIRTUAL_PAD_NAME: &str = "KTRL virtual gamepad";

/// Return the virtual gamepad entity, spawning it on first call.
pub(super) fn ensure(commands: &mut Commands, gamepad: &mut Option<Entity>) -> Entity {
    if let Some(entity) = *gamepad {
        return entity;
    }
    let entity = commands
        .spawn((Gamepad::default(), Name::new(VIRTUAL_PAD_NAME)))
        .id();
    *gamepad = Some(entity);
    entity
}

pub(super) fn parse_button(name: &str) -> Option<GamepadButton> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "south" | "a" => GamepadButton::South,
        "east" | "b" => GamepadButton::East,
        "north" | "y" => GamepadButton::North,
        "west" | "x" => GamepadButton::West,
        "dpadup" | "dpad_up" | "up" => GamepadButton::DPadUp,
        "dpaddown" | "dpad_down" | "down" => GamepadButton::DPadDown,
        "dpadleft" | "dpad_left" | "left" => GamepadButton::DPadLeft,
        "dpadright" | "dpad_right" | "right" => GamepadButton::DPadRight,
        "start" => GamepadButton::Start,
        "select" | "back" => GamepadButton::Select,
        "leftthumb" | "l3" => GamepadButton::LeftThumb,
        "rightthumb" | "r3" => GamepadButton::RightThumb,
        "lefttrigger" | "lt" | "l1" => GamepadButton::LeftTrigger,
        "righttrigger" | "rt" | "r1" => GamepadButton::RightTrigger,
        "lefttrigger2" | "lt2" | "l2" => GamepadButton::LeftTrigger2,
        "righttrigger2" | "rt2" | "r2" => GamepadButton::RightTrigger2,
        _ => return None,
    })
}

pub(super) fn parse_axis(name: &str) -> Option<GamepadAxis> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "leftstickx" | "left_stick_x" | "lx" => GamepadAxis::LeftStickX,
        "leftsticky" | "left_stick_y" | "ly" => GamepadAxis::LeftStickY,
        "rightstickx" | "right_stick_x" | "rx" => GamepadAxis::RightStickX,
        "rightsticky" | "right_stick_y" | "ry" => GamepadAxis::RightStickY,
        "leftz" | "lz" => GamepadAxis::LeftZ,
        "rightz" | "rz" => GamepadAxis::RightZ,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::gamepad::{RawGamepadButtonChangedEvent, RawGamepadEvent};

    #[test]
    fn parses_names() {
        assert_eq!(parse_button("DPadDown"), Some(GamepadButton::DPadDown));
        assert_eq!(parse_button("a"), Some(GamepadButton::South));
        assert_eq!(parse_button("nope"), None);
        assert_eq!(parse_axis("left_stick_x"), Some(GamepadAxis::LeftStickX));
        assert_eq!(parse_axis("nope"), None);
    }

    /// Injected raw events reach the `Gamepad` component: `just_pressed` for one
    /// frame, `pressed` until a release event arrives.
    #[test]
    fn raw_button_event_presses_until_released() {
        let mut app = App::new();
        app.add_plugins(bevy::input::InputPlugin);
        let entity = app.world_mut().spawn(Gamepad::default()).id();

        app.add_systems(
            Update,
            move |mut events: MessageWriter<RawGamepadEvent>, mut frame: Local<u32>| {
                *frame += 1;
                let value = match *frame {
                    1 => 1.0,
                    3 => 0.0,
                    _ => return,
                };
                events.write(RawGamepadEvent::Button(RawGamepadButtonChangedEvent::new(
                    entity,
                    GamepadButton::DPadDown,
                    value,
                )));
            },
        );

        let pressed = |app: &App| {
            app.world()
                .entity(entity)
                .get::<Gamepad>()
                .unwrap()
                .pressed(GamepadButton::DPadDown)
        };
        let just_pressed = |app: &App| {
            app.world()
                .entity(entity)
                .get::<Gamepad>()
                .unwrap()
                .just_pressed(GamepadButton::DPadDown)
        };

        app.update(); // writes the press
        assert!(!pressed(&app));

        app.update(); // processing applies the press
        assert!(pressed(&app) && just_pressed(&app));

        app.update(); // writes the release; `pressed` persists for one more frame
        assert!(pressed(&app) && !just_pressed(&app));

        app.update(); // processing applies the release
        assert!(!pressed(&app));
    }
}
