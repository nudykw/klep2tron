use bevy::prelude::*;
use crate::{GameState, ProgressBar, LoadingEntity};

#[derive(Resource, Default)]
pub struct ClientAssets {
    pub cube_mesh: Handle<Mesh>,
    pub wedge_mesh: Handle<Mesh>,
    pub font: Handle<Font>,
    pub highlight_material: Handle<StandardMaterial>,
}

pub fn start_loading(
    mut commands: Commands, 
    mut assets: ResMut<ClientAssets>, 
    asset_server: Res<AssetServer>,
    state: Res<State<GameState>>,
) {
    assets.cube_mesh = asset_server.load("3dModels/Room/Bricks/cube.obj");
    assets.wedge_mesh = asset_server.load("3dModels/Room/Bricks/wedge.obj");
    assets.font = asset_server.load("fonts/Roboto-Regular.ttf");
    
    if *state.get() == GameState::Loading {
        commands.spawn((Camera2dBundle::default(), LoadingEntity));

        commands.spawn((NodeBundle {
        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        background_color: Color::srgb(0.0, 0.0, 0.0).into(),
        ..default()
    }, LoadingEntity)).with_children(|p| {
        p.spawn(TextBundle::from_section("Loading...", TextStyle { font: assets.font.clone(), font_size: 30.0, color: Color::WHITE }));
        p.spawn((NodeBundle {
            style: Style { width: Val::Px(400.0), height: Val::Px(20.0), border: UiRect::all(Val::Px(2.0)), margin: UiRect::all(Val::Px(20.0)), ..default() },
            border_color: Color::WHITE.into(),
            ..default()
        },)).with_children(|p| {
            p.spawn((NodeBundle {
                style: Style { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() },
                background_color: Color::srgb(0.0, 1.0, 1.0).into(),
                ..default()
            }, ProgressBar));
        });
        });
    }
}

pub fn check_loading_system(
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    assets: Res<ClientAssets>,
    mut bar_query: Query<&mut Style, With<ProgressBar>>,
) {
    use bevy::asset::RecursiveDependencyLoadState;
    let cube_state = asset_server.get_recursive_dependency_load_state(&assets.cube_mesh);
    let wedge_state = asset_server.get_recursive_dependency_load_state(&assets.wedge_mesh);

    let mut loaded_count = 0;
    if cube_state == Some(RecursiveDependencyLoadState::Loaded) { loaded_count += 1; }
    if wedge_state == Some(RecursiveDependencyLoadState::Loaded) { loaded_count += 1; }

    let progress = (loaded_count as f32 / 2.0) * 100.0;

    if let Ok(mut style) = bar_query.get_single_mut() {
        style.width = Val::Percent(progress);
    }

    if loaded_count == 2 {
        next_state.set(GameState::InGame);
    }
}
