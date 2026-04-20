use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use serde::{Deserialize, Serialize};

pub mod ui;
pub mod rendering;
pub mod assets;
pub mod world;
pub mod perf;
pub mod transition;
pub mod input;
pub mod history;

// Re-export key types for convenience
pub use crate::ui::menu::*;
pub use crate::ui::help::*;
pub use crate::ui::hud::*;
pub use crate::assets::*;
pub use crate::rendering::*;
pub use crate::world::*;
pub use crate::perf::*;
pub use crate::transition::*;
pub use crate::input::*;
pub use crate::history::*;

// --- Core Data Structures (The "Project Contract") ---

#[derive(Resource, Default)]
pub struct TileMap {
    pub entities: std::collections::HashMap<(usize, usize), Vec<Entity>>,
}

#[derive(Resource, Default)]
pub struct DirtyTiles {
    pub tiles: Vec<(usize, usize)>,
    pub full_rebuild: bool,
}

#[derive(Resource, Default)]
pub struct Selection {
    pub x: usize,
    pub z: usize,
}

#[derive(Resource, Default)]
pub struct EditorMode {
    pub is_active: bool,
}

#[derive(Resource, Default)]
pub struct HelpState {
    pub is_open: bool,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Loading,
    InGame,
}

#[derive(Component)]
pub struct MenuEntity;

#[derive(Component)]
pub struct LoadingEntity;

#[derive(Component)]
pub struct ProgressBar;

#[derive(Component)]
pub struct HudText;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct ClientCoreOptions {
    pub title: String,
}

impl Default for ClientCoreOptions {
    fn default() -> Self {
        Self { title: "Klep2Tron".to_string() }
    }
}

// --- Data Structures Shared with Server ---
#[derive(Resource, Serialize, Deserialize, Clone, Default, Debug)]
pub struct Project {
    pub rooms: Vec<Room>,
    pub current_room_idx: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub cells: [[Cell; 16]; 16],
}

impl Default for Room {
    fn default() -> Self {
        let mut cells = [[Cell::default(); 16]; 16];
        for x in 0..16 {
            for z in 0..16 {
                cells[x][z].h = 0;
                cells[x][z].tt = TileType::Cube;
            }
        }
        Self { cells }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TileType {
    #[default]
    Empty,
    Cube,
    WedgeN, WedgeE, WedgeS, WedgeW,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default)]
pub struct Cell {
    pub h: i32,
    pub tt: TileType,
}

// --- Plugin Implementation ---
pub struct ClientCorePlugin {
    pub options: ClientCoreOptions,
}

impl Plugin for ClientCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.options.clone())
           .add_plugins(FrameTimeDiagnosticsPlugin)
           .init_state::<GameState>()
           .init_resource::<Project>()
           .init_resource::<ClientAssets>()
           .add_plugins(MenuPlugin)
           .init_resource::<ExtraMenuButtons>()
           .init_resource::<Selection>()
           .init_resource::<EditorMode>()
           .init_resource::<TileMap>()
           .init_resource::<DirtyTiles>()
           .init_resource::<PerfHistory>()
           .init_resource::<HelpState>()
           .init_resource::<RoomTransition>()
           .init_resource::<CommandHistory>()
           .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
           .add_systems(OnEnter(GameState::Menu), setup_menu)
           .add_systems(Update, (
                hud_update_system.run_if(in_state(GameState::InGame)),
                map_rendering_system.run_if(in_state(GameState::InGame)),
                collect_perf_system,
                help_toggle_system,
                help_ui_system,
                transition_logic_system,
                transition_ui_system,
                fullscreen_toggle_system,
           ))
           .add_systems(OnEnter(GameState::Loading), start_loading)
           .add_systems(Update, check_loading_system.run_if(in_state(GameState::Loading)))
           .add_systems(OnExit(GameState::Loading), (cleanup_loading, setup_game_world))
           .add_systems(OnExit(GameState::Menu), cleanup_menu)
           .add_systems(OnExit(GameState::InGame), cleanup_game)
           .add_systems(Last, save_perf_history);
    }
}

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in query.iter() { commands.entity(entity).despawn_recursive(); }
}

pub fn cleanup_loading(mut commands: Commands, query: Query<Entity, With<LoadingEntity>>) {
    for entity in query.iter() { commands.entity(entity).despawn_recursive(); }
}

pub fn cleanup_game(mut commands: Commands, query: Query<Entity, With<MapEntity>>) {
    for entity in query.iter() { commands.entity(entity).despawn_recursive(); }
}
