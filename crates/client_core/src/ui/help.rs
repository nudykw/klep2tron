use bevy::prelude::*;
use crate::ui::widgets::*;
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
                UiNode { node: Node {
                        width: Val::Percent(100.0), height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                        ..default()
                    }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)), global_z_index: GlobalZIndex(100), ..default() },
                HelpUi,
            )).with_children(|parent| {
                parent.spawn(UiNode { node: Node {
                        width: Val::Px(500.0), padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column, row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    }, background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.0)), border_color: BorderColor::all(Color::WHITE), ..default() }).with_children(|p| {
                    p.spawn((ui_text("KLEP2TRON HELP", &font.clone(), 32.0, Color::srgb(0.0, 1.0, 1.0)), Node { margin: UiRect::bottom(Val::Px(20.0)), align_self: AlignSelf::Center, ..default() }));

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
                        p.spawn(UiNode { node: Node { justify_content: JustifyContent::SpaceBetween, ..default() }, ..default() }).with_children(|row| {
                            row.spawn(ui_text(key, &font.clone(), 20.0, Color::srgb(1.0, 1.0, 0.0)));
                            row.spawn(ui_text(desc, &font.clone(), 20.0, Color::WHITE));
                        });
                    }

                    p.spawn((ui_text("Press F1 or Esc to Close", &font.clone(), 16.0, Color::srgb(0.6, 0.6, 0.6)), Node { margin: UiRect::top(Val::Px(20.0)), align_self: AlignSelf::Center, ..default() }));
                });
            });
        }
    } else {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
