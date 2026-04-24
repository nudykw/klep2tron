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
pub mod settings;
pub mod benchmark;
pub mod actor_editor;

#[cfg(not(target_arch = "wasm32"))]
use bevy::winit::WinitWindows;
#[cfg(not(target_arch = "wasm32"))]
use winit::window::Icon;

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
pub use crate::settings::*;
pub use crate::actor_editor::*;

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
    pub is_open:    bool,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Loading,
    InGame,
    Benchmark,
    ActorEditor,
}

#[derive(Component)]
pub struct ActorEditorEntity;



#[derive(Component)]
pub struct LoadingEntity;



#[derive(Component)]
pub struct ProgressBar;

#[derive(Component)]
pub struct HudText;

#[derive(Resource)]
struct AppIconHandle {
    handle: Handle<Image>,
    retries: u32,
}

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
#[derive(Default)]
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
           .init_state::<MenuSubState>()
           .add_plugins(SettingsPlugin)
           .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
           .add_plugins(MaterialPlugin::<StarrySkyMaterial>::default())
           .add_plugins(benchmark::BenchmarkPlugin)
           .add_plugins(actor_editor::ActorEditorPlugin)
           .init_resource::<ExitConfirmationActive>()
           .add_systems(OnEnter(GameState::Menu), setup_menu)
           .add_systems(Startup, (load_app_icon, setup_starry_sky))
           .add_systems(Update, (
                set_window_icon,
                hud_update_system.run_if(in_state(GameState::InGame)),
                map_rendering_system.run_if(in_state(GameState::InGame).or_else(in_state(GameState::Benchmark))),
                collect_perf_system,
                help_toggle_system,
                help_ui_system,
                transition_logic_system,
                transition_ui_system,
                fullscreen_toggle_system,
                apply_graphics_quality_system,
                global_input_system,
                exit_confirmation_sync_system,
                starry_sky_follow_system,
           ))

           .add_systems(OnEnter(GameState::Loading), start_loading)
            .add_systems(OnEnter(GameState::Benchmark), start_loading)
           .add_systems(Update, check_loading_system.run_if(in_state(GameState::Loading)))
           .add_systems(OnExit(GameState::Loading), (cleanup_loading, setup_game_world))
           .add_systems(OnExit(GameState::Menu), cleanup_menu)
           .add_systems(OnExit(GameState::InGame), (cleanup_game, cleanup_menu, reset_ambient_light))
           .add_systems(Last, save_perf_history)
            .add_systems(Update, finish_loading_settings_on_menu.run_if(in_state(GameState::Menu)));
    }
}

fn global_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    editor_mode: Res<EditorMode>,
    mut exit_confirm: ResMut<ExitConfirmationActive>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match *state.get() {
            GameState::InGame => {
                if editor_mode.is_active {
                    next_state.set(GameState::Menu);
                } else {
                    exit_confirm.0 = !exit_confirm.0;
                }
            },
            _ => {}
        }
    }
}

fn exit_confirmation_sync_system(
    mut commands: Commands,
    exit_confirm: Res<ExitConfirmationActive>,
    game_state: Res<State<GameState>>,
    mut confirmation: ResMut<ConfirmationData>,
    mut next_menu_state: ResMut<NextState<MenuSubState>>,
    menu_query: Query<Entity, With<MenuEntity>>,
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<Camera2d>>,
) {
    if *game_state.get() != GameState::InGame { return; }
    
    if exit_confirm.is_changed() {
        if exit_confirm.0 {
            // Setup confirmation UI
            confirmation.message = "Quit to main menu?".to_string();
            confirmation.has_cancel = false;
            next_menu_state.set(MenuSubState::Confirmation);
            setup_menu(commands, asset_server, game_state, exit_confirm, camera_query, next_menu_state);
        } else {
            // Cleanup confirmation UI
            for entity in menu_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}




fn finish_loading_settings_on_menu(mut settings: ResMut<GraphicsSettings>) {
    if settings.is_loading {
        settings.is_loading = false;
        save_settings(&settings);
    }
}



pub fn cleanup_loading(mut commands: Commands, query: Query<Entity, With<LoadingEntity>>) {
    for entity in query.iter() { commands.entity(entity).despawn_recursive(); }
}

pub fn reset_ambient_light(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
    });
}

pub fn cleanup_game(
    mut commands: Commands, 
    query: Query<Entity, Or<(With<MapEntity>, With<TileEntity>)>>,
    mut tile_map: ResMut<TileMap>,
) {
    for entity in query.iter() { 
        if let Some(e) = commands.get_entity(entity) {
            e.despawn_recursive(); 
        }
    }
    tile_map.entities.clear();
}

#[cfg(not(target_arch = "wasm32"))]
fn load_app_icon(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(AppIconHandle {
        handle: asset_server.load("icons/app_icon.png"),
        retries: 3,
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    images: Res<Assets<Image>>,
    icon_handle: Option<ResMut<AppIconHandle>>,
    mut commands: Commands,
) {
    let Some(mut icon_handle) = icon_handle else { return; };

    // Wayland check
    let is_wayland = std::env::var("XDG_SESSION_TYPE").map(|v| v == "wayland").unwrap_or(false);
    if is_wayland {
        commands.remove_resource::<AppIconHandle>();
        return;
    }
    
    if let Some(image) = images.get(&icon_handle.handle) {
        let Ok(dynamic_image) = image.clone().try_into_dynamic() else {
            warn!("Failed to convert icon image to dynamic image");
            commands.remove_resource::<AppIconHandle>();
            return;
        };
        let rgba_image = dynamic_image.into_rgba8();
        let (width, height) = rgba_image.dimensions();
        let rgba = rgba_image.into_raw();
        
        info!("Setting window icon: {}x{}, buffer size: {}", width, height, rgba.len());

        let Ok(icon) = Icon::from_rgba(rgba, width, height) else {
            warn!("Failed to create icon from rgba");
            commands.remove_resource::<AppIconHandle>();
            return;
        };

        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
        
        if icon_handle.retries > 0 {
            icon_handle.retries -= 1;
        } else {
            commands.remove_resource::<AppIconHandle>();
            info!("Finished setting window icon");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_app_icon() {}
#[cfg(target_arch = "wasm32")]
fn set_window_icon() {}
