//! `POST /text` — inject text into focused text fields.
//!
//! Text widgets (e.g. the actor editor) read [`KeyboardInput`], so we synthesize
//! one pressed event per character with the right `logical_key`
//! (`Key::Character` / `Key::Space` / `Key::Backspace` / `Key::Enter`).

use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKeyCode};
use bevy::input::ButtonState;
use bevy::prelude::*;

use super::http::Response;

pub(super) fn respond(
    text: &str,
    window: Option<Entity>,
    keyboard: &mut MessageWriter<KeyboardInput>,
) -> Response {
    let Some(window) = window else {
        return Response::text(503, "no primary window".into());
    };
    let count = inject_text(text, keyboard, window);
    Response::json(format!("{{\"ok\":true,\"chars\":{count}}}"))
}

fn inject_text(text: &str, keyboard: &mut MessageWriter<KeyboardInput>, window: Entity) -> usize {
    let mut count = 0;
    for ch in text.chars() {
        let (key_code, logical_key, raw): (KeyCode, Key, Option<String>) = match ch {
            '\n' => (KeyCode::Enter, Key::Enter, None),
            '\t' => (KeyCode::Tab, Key::Tab, None),
            '\u{8}' => (KeyCode::Backspace, Key::Backspace, None),
            ' ' => (KeyCode::Space, Key::Space, Some(" ".to_string())),
            _ => (
                KeyCode::Unidentified(NativeKeyCode::Unidentified),
                Key::Character(ch.to_string().into()),
                Some(ch.to_string()),
            ),
        };
        keyboard.write(KeyboardInput {
            key_code,
            logical_key,
            state: ButtonState::Pressed,
            text: raw.map(|s| s.into()),
            repeat: false,
            window,
        });
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct Captured(Vec<KeyboardInput>);

    #[test]
    fn synthesizes_one_event_per_character() {
        let mut app = App::new();
        app.add_message::<KeyboardInput>();
        app.init_resource::<Captured>();
        let window = app.world_mut().spawn_empty().id();

        app.add_systems(
            Update,
            (
                move |mut writer: MessageWriter<KeyboardInput>| {
                    inject_text("a b", &mut writer, window);
                },
                |mut reader: MessageReader<KeyboardInput>, mut captured: ResMut<Captured>| {
                    for event in reader.read() {
                        captured.0.push(event.clone());
                    }
                },
            )
                .chain(),
        );
        app.update();

        let captured = app.world().resource::<Captured>();
        assert_eq!(captured.0.len(), 3);
        assert_eq!(captured.0[0].logical_key, Key::Character("a".into()));
        assert_eq!(captured.0[1].logical_key, Key::Space);
        assert_eq!(captured.0[2].logical_key, Key::Character("b".into()));
        assert!(captured.0.iter().all(|e| e.state.is_pressed()));
    }
}
