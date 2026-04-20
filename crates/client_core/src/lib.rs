use bevy::prelude::*;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use serde::{Deserialize, Serialize};

// --- Core Data Structures ---

#[derive(Resource, Default)]
pub struct Selection {
    pub x: usize,
    pub z: usize,
}

#[derive(Resource, Default)]
pub struct HelpState {
    pub is_open: bool,
}

#[derive(Component)]
pub struct HelpUi;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum TransitionPhase {
    #[default] Idle,
    Out,
    In,
}

#[derive(Resource, Default)]
pub struct RoomTransition {
    pub phase: TransitionPhase,
    pub timer: f32,
    pub target_room_idx: usize,
    pub speed: f32, // 1.0 / duration in seconds
}

impl RoomTransition {
    pub fn start(&mut self, target: usize) {
        if self.phase == TransitionPhase::Idle {
            self.phase = TransitionPhase::Out;
            self.timer = 0.0;
            self.target_room_idx = target;
            self.speed = 4.0; // 0.25s out, 0.25s in
        }
    }
}

#[derive(Component)]
pub struct TransitionUi;

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize, Component)]
pub enum TileType {
    #[default] Empty,
    Cube,
    WedgeN, WedgeE, WedgeS, WedgeW,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct TileCell {
    pub h: i32,
    pub tt: TileType,
}

#[derive(Serialize, Deserialize, Clone, Resource, Default)]
pub struct Room {
    pub cells: [[TileCell; 16]; 16],
}

#[derive(Resource, Serialize, Deserialize, Default, Clone)]
pub struct Project {
    pub rooms: Vec<Room>,
    pub current_room_idx: usize,
}

#[derive(Resource, Default)]
pub struct CommandHistory {
    pub undo_stack: Vec<Project>,
    pub redo_stack: Vec<Project>,
}

impl CommandHistory {
    pub fn push_undo(&mut self, project: &Project) {
        self.undo_stack.push(project.clone());
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: &Project) -> Option<Project> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current.clone());
        Some(prev)
    }

    pub fn redo(&mut self, current: &Project) -> Option<Project> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current.clone());
        Some(next)
    }
}

#[derive(Resource, Default)]
pub struct ClientAssets {
    pub cube_mesh: Handle<Mesh>,
    pub wedge_mesh: Handle<Mesh>,
    pub font: Handle<Font>,
    pub highlight_material: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct TileMap {
    pub entities: std::collections::HashMap<(usize, usize), Vec<Entity>>,
}

#[derive(Resource, Default)]
pub struct DirtyTiles {
    pub tiles: Vec<(usize, usize)>,
    pub full_rebuild: bool,
}

#[derive(Resource, Default, serde::Serialize)]
pub struct PerfHistory {
    pub entries: Vec<PerfEntry>,
}

#[derive(serde::Serialize, Clone)]
pub struct PerfEntry {
    pub timestamp: f64,
    pub fps: f32,
    pub cpu: f32,
    pub mem: f32,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default] Menu,
    Loading,
    InGame,
}

// --- Components ---

#[derive(Component)] pub struct MenuEntity;
#[derive(Component)] pub struct LoadingEntity;
#[derive(Component)] pub struct ProgressBar;
#[derive(Component)] pub struct MapEntity;
#[derive(Component)] pub struct TileEntity;

// --- Plugin ---

#[derive(Component)]
pub struct HudText;

#[derive(Resource, Default, Clone)]
pub struct ClientCoreOptions {
    pub skip_default_setup: bool,
}

pub struct ClientCorePlugin {
    pub options: ClientCoreOptions,
}

impl Default for ClientCorePlugin {
    fn default() -> Self {
        Self { options: ClientCoreOptions::default() }
    }
}

impl Plugin for ClientCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
           .insert_resource(self.options.clone())
           .init_resource::<Project>()
           .init_resource::<ClientAssets>()
           .init_resource::<ExtraMenuButtons>()
           .init_resource::<Selection>()
           .init_resource::<TileMap>()
           .init_resource::<DirtyTiles>()
           .init_resource::<PerfHistory>()
           .init_resource::<CommandHistory>()
           .init_resource::<HelpState>()
           .init_resource::<RoomTransition>()
           .add_plugins(FrameTimeDiagnosticsPlugin)
           .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
           .add_systems(OnEnter(GameState::Menu), setup_menu)
           .add_systems(Update, (
                menu_system.run_if(in_state(GameState::Menu)),
                hud_update_system.run_if(in_state(GameState::InGame)),
                map_rendering_system.run_if(in_state(GameState::InGame)),
                collect_perf_system,
                help_input_system,
                help_ui_system,
                transition_logic_system,
                transition_ui_system,
            ))
           .add_systems(PostUpdate, save_perf_history)
           .add_systems(OnEnter(GameState::Loading), start_loading)
           .add_systems(Update, check_loading_system.run_if(in_state(GameState::Loading)))
           .add_systems(OnExit(GameState::Menu), cleanup_menu)
           .add_systems(OnEnter(GameState::InGame), cleanup_loading);

        if !self.options.skip_default_setup {
            app.add_systems(OnEnter(GameState::InGame), setup_game_world);
        }

        app.add_systems(Update, map_rendering_system.run_if(in_state(GameState::InGame)));
    }
}

// --- Systems ---

pub fn setup_menu(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    extra_buttons: Res<ExtraMenuButtons>
) {
    commands.spawn((Camera2dBundle::default(), MenuEntity));
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    
    commands.spawn((NodeBundle {
        style: Style {
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            display: Display::Flex, flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            row_gap: Val::Px(20.0),
            ..default()
        },
        background_color: Color::srgb(0.01, 0.01, 0.05).into(),
        ..default()
    }, MenuEntity)).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Klep2tron",
            TextStyle { font: font.clone(), font_size: 100.0, color: Color::WHITE },
        ).with_style(Style { margin: UiRect::bottom(Val::Px(40.0)), ..default() }));

        // Start Game Button (Default)
        spawn_menu_button(parent, &font, "START GAME", MenuAction::StartGame);

        // Extra Buttons (e.g. from Editor)
        for (label, action) in extra_buttons.buttons.iter() {
            spawn_menu_button(parent, &font, label, action.clone());
        }
    });
}

#[derive(Resource, Default)]
pub struct ExtraMenuButtons {
    pub buttons: Vec<(String, MenuAction)>,
}

#[derive(Component, Clone)]
pub enum MenuAction {
    StartGame,
    StartEditor,
}

fn spawn_menu_button(parent: &mut ChildBuilder, font: &Handle<Font>, text: &str, action: MenuAction) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(250.0), height: Val::Px(60.0),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                ..default()
            },
            border_color: Color::WHITE.into(),
            background_color: Color::srgb(0.2, 0.2, 0.2).into(),
            ..default()
        },
        action,
    )).with_children(|p| {
        p.spawn(TextBundle::from_section(
            text,
            TextStyle { font: font.clone(), font_size: 30.0, color: Color::WHITE },
        ));
    });
}

pub fn menu_system(
    mut interaction_query: Query<(&Interaction, &MenuAction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, action, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match action {
                    MenuAction::StartGame => next_state.set(GameState::Loading),
                    MenuAction::StartEditor => next_state.set(GameState::Loading),
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.3, 0.3, 0.3).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.2, 0.2).into();
            }
        }
    }
}

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in query.iter() { commands.entity(entity).despawn_recursive(); }
}

pub fn start_loading(mut commands: Commands, mut assets: ResMut<ClientAssets>, asset_server: Res<AssetServer>) {
    assets.cube_mesh = asset_server.load("3dModels/Room/Bricks/cube.obj");
    assets.wedge_mesh = asset_server.load("3dModels/Room/Bricks/wedge.obj");
    assets.font = asset_server.load("fonts/Roboto-Regular.ttf");

    // Spawn Loading Camera
    commands.spawn((Camera2dBundle::default(), LoadingEntity));

    // Spawn Progress Bar UI
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

pub fn cleanup_loading(mut commands: Commands, query: Query<Entity, With<LoadingEntity>>) {
    for entity in query.iter() { commands.entity(entity).despawn_recursive(); }
}

pub fn setup_game_world(mut commands: Commands, mut project: ResMut<Project>, asset_server: Res<AssetServer>) {
    // Load map.json
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(content) = std::fs::read_to_string("assets/map.json") {
        if let Ok(loaded) = serde_json::from_str::<Project>(&content) {
            *project = loaded;
        }
    }
    
    if project.rooms.is_empty() { 
        project.rooms.push(Room::default()); 
    }

    // Spawning a default 3D camera for the game client
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(20.0, 15.0, 20.0).looking_at(Vec3::new(7.5, 0.0, 7.5), Vec3::Y),
        ..default()
    }, MapEntity));

    commands.insert_resource(AmbientLight { color: Color::WHITE, brightness: 500.0 });

    // HUD for the client
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.spawn((NodeBundle {
        style: Style { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), padding: UiRect::all(Val::Px(8.0)), flex_direction: FlexDirection::Column, ..default() },
        background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(), ..default()
    }, MapEntity)).with_children(|p| {
        p.spawn((TextBundle::from_section("FPS: 0", TextStyle { font, font_size: 18.0, color: Color::WHITE }), HudText));
    });
}

#[derive(Resource, Default)]
pub struct MaterialCache {
    pub map: std::collections::HashMap<(i32, bool), (Handle<StandardMaterial>, Handle<StandardMaterial>)>,
}

pub fn map_rendering_system(
    mut commands: Commands,
    project: Res<Project>,
    assets: Res<ClientAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tile_query: Query<Entity, With<TileEntity>>,
    mut cache: Local<MaterialCache>,
    mut first_run: Local<bool>,
    mut last_room: Local<Option<usize>>,
    mut tile_map: ResMut<TileMap>,
    mut dirty: ResMut<DirtyTiles>,
) {
    if assets.cube_mesh == Handle::default() { return; } // Wait for assets
    let room_changed = last_room.map_or(true, |r| r != project.current_room_idx);
    
    if !*first_run || room_changed || dirty.full_rebuild || project.is_changed() {
        if !*first_run || room_changed || dirty.full_rebuild {
            *first_run = true;
            *last_room = Some(project.current_room_idx);
            dirty.full_rebuild = false;
            dirty.tiles.clear();
            
            // Full rebuild: despawn everything and mark all cells for spawning
            for entities in tile_map.entities.values() {
                for &entity in entities { commands.entity(entity).despawn_recursive(); }
            }
            tile_map.entities.clear();
            for x in 0..16 { for z in 0..16 { dirty.tiles.push((x, z)); } }
        } else if project.is_changed() {
            // Project changed externally (e.g. loaded), mark all tiles for update
            for x in 0..16 { for z in 0..16 { dirty.tiles.push((x, z)); } }
        }
    } else if dirty.tiles.is_empty() {
        return;
    }
    
    if project.rooms.is_empty() { return; }
    let room = &project.rooms[project.current_room_idx];
    
    // Process only dirty tiles
    let tiles_to_process = std::mem::take(&mut dirty.tiles);
    for (x, z) in tiles_to_process {
        // Despawn old entities at this coordinate
        if let Some(entities) = tile_map.entities.remove(&(x, z)) {
            for entity in entities { commands.entity(entity).despawn_recursive(); }
        }
        
        let cell = room.cells[x][z];
        if cell.h < 0 { continue; }
        let is_even = (x + z) % 2 == 0;

        // --- Same rendering logic as before ---
        let mut tx = x as i32;
        let mut tz = z as i32;
        let mut steps = 0;
        let mut found_cube_h = None;
        while steps < 16 {
            let c = room.cells[tx as usize][tz as usize];
            if matches!(c.tt, TileType::Cube) { found_cube_h = Some(c.h); break; }
            let (dx, dz) = match c.tt {
                TileType::WedgeN => (0, -1), TileType::WedgeE => (1, 0),
                TileType::WedgeS => (0, 1), TileType::WedgeW => (-1, 0),
                _ => break,
            };
            tx += dx; tz += dz;
            if tx < 0 || tx >= 16 || tz < 0 || tz >= 16 { break; }
            steps += 1;
        }
        let target_h = found_cube_h.unwrap_or(cell.h);

        let (mat_top, mat_side) = cache.map.entry((target_h, is_even)).or_insert_with(|| {
            let h_val = (target_h as f32 * 0.1).clamp(0.0, 1.0);
            let base_color = if is_even { Color::srgb(0.4+h_val*0.2, 0.7+h_val*0.2, 0.2) } 
                                 else { Color::srgb(0.3+h_val*0.2, 0.6+h_val*0.2, 0.1) };
            let side_color = Color::from(LinearRgba::from(base_color) * 0.7);
            (materials.add(StandardMaterial { base_color, ..default() }), materials.add(StandardMaterial { base_color: side_color, ..default() }))
        }).clone();

        let (mesh, rot) = match cell.tt {
            TileType::Cube => (assets.cube_mesh.clone(), 0.0),
            TileType::WedgeN => (assets.wedge_mesh.clone(), 0.0),
            TileType::WedgeE => (assets.wedge_mesh.clone(), -std::f32::consts::FRAC_PI_2),
            TileType::WedgeS => (assets.wedge_mesh.clone(), std::f32::consts::PI),
            TileType::WedgeW => (assets.wedge_mesh.clone(), std::f32::consts::FRAC_PI_2),
            _ => (assets.cube_mesh.clone(), 0.0),
        };

        let h_val = cell.h as f32;
        let mut entities = Vec::new();
        let top_id = commands.spawn((PbrBundle {
            mesh, material: mat_top,
            transform: Transform::from_translation(Vec3::new(x as f32, h_val * 0.5 - 0.25, z as f32))
                .with_scale(Vec3::new(1.0, 0.5, 1.0)).with_rotation(Quat::from_rotation_y(rot)),
            ..default()
        }, TileEntity)).id();
        entities.push(top_id);

        if cell.h > 1 {
            let column_h = (h_val - 1.0) * 0.5;
            let col_id = commands.spawn((PbrBundle {
                mesh: assets.cube_mesh.clone(), material: mat_side.clone(),
                transform: Transform::from_translation(Vec3::new(x as f32, column_h * 0.5, z as f32))
                    .with_scale(Vec3::new(1.0, column_h, 1.0)),
                ..default()
            }, TileEntity)).id();
            entities.push(col_id);
        }
        tile_map.entities.insert((x, z), entities);
    }
}

pub fn hud_update_system(
    diagnostics: Res<DiagnosticsStore>, 
    project: Res<Project>, 
    selection: Option<Res<Selection>>, 
    mut query: Query<&mut Text, With<HudText>>
) {
    let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d: &bevy::diagnostic::Diagnostic| d.smoothed()).unwrap_or(0.0);
    for mut text in query.iter_mut() {
        if let Some(sel) = selection.as_ref() {
            if project.current_room_idx < project.rooms.len() {
                let room = &project.rooms[project.current_room_idx];
                let cell = room.cells[sel.x][sel.z];
                text.sections[0].value = format!("FPS: {:.0} | ROOM: {} | POS: {}, {}, {}", fps, project.current_room_idx, sel.x, sel.z, cell.h);
            }
        } else {
            text.sections[0].value = format!("FPS: {:.0} | ROOM: {}", fps, project.current_room_idx);
        }
    }
}

pub fn collect_perf_system(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut history: ResMut<PerfHistory>,
) {
    let current_time = time.elapsed_seconds_f64();
    if history.entries.last().map_or(true, |e| (current_time - e.timestamp) >= 1.0) {
        let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d| d.smoothed()).unwrap_or(0.0) as f32;
        let cpu = diagnostics.get(&bevy::diagnostic::SystemInformationDiagnosticsPlugin::CPU_USAGE).and_then(|d| d.smoothed()).unwrap_or(0.0) as f32;
        let mem = diagnostics.get(&bevy::diagnostic::SystemInformationDiagnosticsPlugin::MEM_USAGE).and_then(|d| d.smoothed()).unwrap_or(0.0) as f32;
        
        history.entries.push(PerfEntry {
            timestamp: current_time,
            fps, cpu, mem
        });
    }
}

pub fn save_perf_history(
    history: Res<PerfHistory>,
    mut exit_events: EventReader<bevy::app::AppExit>,
) {
    for _ in exit_events.read() {
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(json) = serde_json::to_string_pretty(&*history) {
            let _ = std::fs::write("perf_metrics.json", json);
            println!("--- Performance Report Saved to perf_metrics.json ---");
        }
    }
}

pub fn help_input_system(keyboard: Res<ButtonInput<KeyCode>>, mut help_state: ResMut<HelpState>) {
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
                        // HOTKEY_SYNC: Keep this list in sync with actual input systems
                        ("F1 / Esc", "Toggle / Close Help"),
                        ("Ctrl+Z / Ctrl+U", "Undo / Redo"),
                        ("Arrows", "Move Selection"),
                        ("Shift + Arrows", "Camera Orbit / Zoom"),
                        ("Q / A", "Change Height / Up-Down"),
                        ("F", "Clone Neighbor Tile"),
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

pub fn transition_logic_system(
    time: Res<Time>,
    mut transition: ResMut<RoomTransition>,
    mut project: ResMut<Project>,
    mut dirty: ResMut<DirtyTiles>,
) {
    if transition.phase == TransitionPhase::Idle { return; }

    transition.timer += time.delta_seconds() * transition.speed;

    if transition.phase == TransitionPhase::Out && transition.timer >= 1.0 {
        // Switch room at peak darkness
        project.current_room_idx = transition.target_room_idx;
        dirty.full_rebuild = true;
        transition.phase = TransitionPhase::In;
        transition.timer = 0.0;
    } else if transition.phase == TransitionPhase::In && transition.timer >= 1.0 {
        transition.phase = TransitionPhase::Idle;
        transition.timer = 0.0;
    }
}

pub fn transition_ui_system(
    mut commands: Commands,
    transition: Res<RoomTransition>,
    query: Query<Entity, With<TransitionUi>>,
    mut overlay_query: Query<&mut BackgroundColor, With<TransitionUi>>,
) {
    if transition.phase == TransitionPhase::Idle {
        if let Ok(entity) = query.get_single() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    if query.is_empty() {
        commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0), height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: Color::NONE.into(),
                z_index: ZIndex::Global(1000), // Always on top
                ..default()
            },
            TransitionUi,
        ));
    } else if let Ok(mut color) = overlay_query.get_single_mut() {
        let alpha = match transition.phase {
            TransitionPhase::Out => transition.timer.clamp(0.0, 1.0),
            TransitionPhase::In => (1.0 - transition.timer).clamp(0.0, 1.0),
            _ => 0.0,
        };
        *color = Color::srgba(0.0, 0.0, 0.0, alpha).into();
    }
}
