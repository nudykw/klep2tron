use bevy::prelude::*;
use crate::HelpState;

#[derive(Component)]
pub struct HelpUi;

pub fn help_toggle_system(keyboard: Res<ButtonInput<KeyCode>>, mut help_state: ResMut<HelpState>) {
    if keyboard.just_pressed(KeyCode::F1) {
        help_state.is_open = !help_state.is_open;
    }
    if keyboard.just_pressed(KeyCode::Escape) && help_state.is_open {
        help_state.is_open = false;
    }
}

pub fn help_ui_system(
    mut commands: Commands,
    help_state: Res<HelpState>,
    query: Query<Entity, With<HelpUi>>,
    asset_server: Res<AssetServer>,
) {
    if !help_state.is_changed() { return; }

    if help_state.is_open {
        if query.is_empty() {
            let font = asset_server.load("fonts/Roboto-Regular.ttf");
            commands.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0), height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.85).into(),
                    z_index: ZIndex::Global(100),
                    ..default()
                },
                HelpUi,
            )).with_children(|parent| {
                parent.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(500.0), padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column, row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::srgba(0.1, 0.1, 0.1, 1.0).into(),
                    border_color: Color::WHITE.into(),
                    ..default()
                }).with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "KLEP2TRON HELP",
                        TextStyle { font: font.clone(), font_size: 32.0, color: Color::srgb(0.0, 1.0, 1.0) },
                    ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), align_self: AlignSelf::Center, ..default() }));

                    let controls = [
                        ("F1 / Esc", "Toggle / Close Help"),
                        ("Ctrl+Enter", "Toggle Fullscreen"),
                        ("Ctrl+Z / Ctrl+U", "Undo / Redo"),
                        ("Arrows", "Move Selection"),
                        ("Shift + Arrows", "Camera Orbit / Zoom"),
                        ("Q / A", "Change Height / Up-Down"),
                        ("F", "Clone Previous Selection"),
                        ("[ / ]", "Switch Room"),
                        ("Esc", "Return to Menu"),
                        ("Left Mouse", "Select Tile / Move"),
                    ];

                    for (key, desc) in controls {
                        p.spawn(NodeBundle {
                            style: Style { justify_content: JustifyContent::SpaceBetween, ..default() },
                            ..default()
                        }).with_children(|row| {
                            row.spawn(TextBundle::from_section(key, TextStyle { font: font.clone(), font_size: 20.0, color: Color::srgb(1.0, 1.0, 0.0) }));
                            row.spawn(TextBundle::from_section(desc, TextStyle { font: font.clone(), font_size: 20.0, color: Color::WHITE }));
                        });
                    }

                    p.spawn(TextBundle::from_section(
                        "Press F1 or Esc to Close",
                        TextStyle { font: font.clone(), font_size: 16.0, color: Color::srgb(0.6, 0.6, 0.6) },
                    ).with_style(Style { margin: UiRect::top(Val::Px(20.0)), align_self: AlignSelf::Center, ..default() }));
                });
            });
        }
    } else {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
