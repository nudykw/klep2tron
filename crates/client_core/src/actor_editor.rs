use bevy::prelude::*;
use crate::{ActorEditorEntity, GameState};

pub fn setup_actor_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 3D Camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 5,
                clear_color: Color::BLACK.into(),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    // Light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(2.0, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ActorEditorEntity,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // Base UI Node
    let font = asset_server.load("fonts/Roboto-Regular.ttf");

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        },
        ActorEditorEntity,
    )).with_children(|parent| {
        // Top Toolbar Placeholder
        parent.spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(20.0)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        }, ActorEditorEntity)).with_children(|p| {
            p.spawn(TextBundle::from_section(
                "ACTOR EDITOR - WORK IN PROGRESS",
                TextStyle {
                    font: font.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ));
        });

        // Bottom Status Bar Placeholder
        parent.spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(20.0)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.9).into(),
            ..default()
        }, ActorEditorEntity)).with_children(|p| {
            p.spawn(TextBundle::from_section(
                "Ready",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                },
            ));
        });
        
        // Temporary Back Button for testing
        parent.spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(70.0),
                    right: Val::Px(20.0),
                    width: Val::Px(100.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgb(0.3, 0.1, 0.1).into(),
                ..default()
            },
            ActorEditorBackButton,
        )).with_children(|p| {
            p.spawn(TextBundle::from_section(
                "BACK",
                TextStyle { font: font.clone(), font_size: 20.0, color: Color::WHITE },
            ));
        });
    });
}

#[derive(Component)]
pub struct ActorEditorBackButton;

pub fn cleanup_actor_editor(
    mut commands: Commands,
    query: Query<Entity, With<ActorEditorEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn actor_editor_input_system(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ActorEditorBackButton>)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }

    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Menu);
        }
    }
}
