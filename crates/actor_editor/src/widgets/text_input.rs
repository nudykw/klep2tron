use bevy::prelude::*;
use bevy::input::keyboard::{KeyboardInput, Key};

#[derive(Component)]
pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    pub is_focused: bool,
    pub is_valid: bool,
    pub max_length: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: "Enter text...".to_string(),
            is_focused: false,
            is_valid: true,
            max_length: 32,
        }
    }
}

#[derive(Component)]
pub struct TextInputContent;

#[derive(Bundle)]
pub struct TextInputBundle {
    pub button: ButtonBundle,
    pub input: TextInput,
}

pub fn spawn_text_input(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    placeholder: &str,
    initial_value: &str,
    width: Val,
) -> Entity {
    parent.spawn(TextInputBundle {
        button: ButtonBundle {
            style: Style {
                width,
                height: Val::Px(28.0),
                padding: UiRect::horizontal(Val::Px(8.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        input: TextInput {
            value: initial_value.to_string(),
            placeholder: placeholder.to_string(),
            ..default()
        },
    }).with_children(|p| {
        p.spawn((
            TextBundle::from_section(
                if initial_value.is_empty() { placeholder } else { initial_value },
                TextStyle {
                    font: font.clone(),
                    font_size: 13.0,
                    color: if initial_value.is_empty() { Color::srgb(0.5, 0.5, 0.5) } else { Color::WHITE },
                },
            ),
            TextInputContent,
        ));
    }).id()
}

pub fn text_input_system(
    mut char_events: EventReader<KeyboardInput>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(Entity, &Interaction, &mut TextInput, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut Text, With<TextInputContent>>,
) {
    // 1. Handle focus changes
    for (_entity, interaction, mut input, _, _) in query.iter_mut() {
        if *interaction == Interaction::Pressed {
            input.is_focused = true;
        } else if mouse.just_pressed(MouseButton::Left) {
            if *interaction != Interaction::Hovered && input.is_focused {
                input.is_focused = false;
            }
        }
    }

    // 2. Visual updates
    for (_, interaction, input, mut bg, children) in query.iter_mut() {
        let color = if input.is_focused {
            Color::srgba(0.05, 0.3, 0.5, 0.8) // Sleek Blue Focus
        } else if *interaction == Interaction::Hovered {
            Color::srgba(1.0, 1.0, 1.0, 0.08) // Subtle Hover
        } else if !input.is_valid {
            Color::srgba(0.5, 0.1, 0.1, 0.5)
        } else {
            Color::srgba(0.0, 0.0, 0.0, 0.3)  // Clean Dark
        };
        *bg = color.into();

        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                if input.value.is_empty() && !input.is_focused {
                    text.sections[0].value = input.placeholder.clone();
                    text.sections[0].style.color = Color::srgb(0.4, 0.4, 0.4);
                } else {
                    text.sections[0].value = format!("{}{}", input.value, if input.is_focused { "|" } else { "" });
                    text.sections[0].style.color = Color::WHITE;
                }
            }
        }
    }

    // 3. Handle keyboard for the focused input
    let mut events = Vec::new();
    for event in char_events.read() {
        if event.state.is_pressed() {
            events.push(event.clone());
        }
    }

    if !events.is_empty() {
        for (_, _, mut input, _, _) in query.iter_mut() {
            if input.is_focused {
                for event in &events {
                    match &event.logical_key {
                        Key::Character(c) => {
                            if input.value.len() < input.max_length {
                                input.value.push_str(c.as_str());
                            }
                        }
                        Key::Space => {
                            if input.value.len() < input.max_length {
                                input.value.push(' ');
                            }
                        }
                        Key::Backspace => {
                            input.value.pop();
                        }
                        Key::Enter | Key::Escape => {
                            input.is_focused = false;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
