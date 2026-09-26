use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;
use crate::ui::widgets::*;
use crate::actor::{ActorManifest, PlayerActorConfig};
use crate::{GameState, ProgressBar, LoadingEntity};

#[derive(Resource, Default)]
pub struct ClientAssets {
    pub cube_mesh: Handle<Mesh>,
    pub wedge_mesh: Handle<Mesh>,
    pub font: Handle<Font>,
    pub highlight_material: Handle<StandardMaterial>,
    /// Manifest of the player hero (`actors/<Name>/actor.ron`).
    pub actor_manifest: Handle<ActorManifest>,
    /// Baked head mesh of the player hero.
    pub actor_head: Handle<Mesh>,
    /// Baked body mesh of the player hero.
    pub actor_body: Handle<Mesh>,
    /// Baked engine (legs) mesh of the player hero.
    pub actor_legs: Handle<Mesh>,
}

pub fn start_loading(
    mut commands: Commands, 
    mut assets: ResMut<ClientAssets>, 
    asset_server: Res<AssetServer>,
    state: Res<State<GameState>>,
    actor: Res<PlayerActorConfig>,
) {
    assets.cube_mesh = asset_server.load("3dModels/Room/Bricks/cube.obj");
    assets.wedge_mesh = asset_server.load("3dModels/Room/Bricks/wedge.obj");
    assets.font = asset_server.load("fonts/Roboto-Regular.ttf");

    assets.actor_manifest = asset_server.load(actor.manifest());
    assets.actor_head = asset_server.load(actor.head_mesh());
    assets.actor_body = asset_server.load(actor.body_mesh());
    assets.actor_legs = asset_server.load(actor.legs_mesh());
    
    if *state.get() == GameState::Loading {
        commands.spawn((Camera2d, LoadingEntity));

        commands.spawn((UiNode { node: Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)), ..default() }, LoadingEntity)).with_children(|p| {
        p.spawn(ui_text("Loading...", &assets.font.clone(), 30.0, Color::WHITE));
        p.spawn((UiNode { node: Node { width: Val::Px(400.0), height: Val::Px(20.0), border: UiRect::all(Val::Px(2.0)), margin: UiRect::all(Val::Px(20.0)), ..default() }, border_color: BorderColor::all(Color::WHITE), ..default() },)).with_children(|p| {
            p.spawn((UiNode { node: Node { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() }, background_color: BackgroundColor(Color::srgb(0.0, 1.0, 1.0)), ..default() }, ProgressBar));
        });
        });
    }
}

pub fn check_loading_system(
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    assets: Res<ClientAssets>,
    mut bar_query: Query<&mut Node, With<ProgressBar>>,
) {
    let states = [
        asset_server.get_recursive_dependency_load_state(&assets.cube_mesh),
        asset_server.get_recursive_dependency_load_state(&assets.wedge_mesh),
        asset_server.get_recursive_dependency_load_state(&assets.actor_manifest),
        asset_server.get_recursive_dependency_load_state(&assets.actor_head),
        asset_server.get_recursive_dependency_load_state(&assets.actor_body),
        asset_server.get_recursive_dependency_load_state(&assets.actor_legs),
    ];

    let loaded_count = states.iter().filter(|s| s.as_ref().is_some_and(|s| s.is_loaded())).count();
    let failed_count = states
        .iter()
        .filter(|s| matches!(s.as_ref(), Some(RecursiveDependencyLoadState::Failed(_))))
        .count();

    let progress = (loaded_count as f32 / states.len() as f32) * 100.0;

    if let Ok(mut style) = bar_query.single_mut() {
        style.width = Val::Percent(progress);
    }

    if loaded_count + failed_count == states.len() {
        if failed_count > 0 {
            warn!("{failed_count} asset(s) failed to load; entering the game anyway");
        }
        next_state.set(GameState::InGame);
    }
}
