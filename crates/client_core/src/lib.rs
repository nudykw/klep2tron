use bevy::prelude::*;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use serde::{Deserialize, Serialize};

// --- Core Data Structures ---

#[derive(Resource, Default)]
pub struct Selection {
    pub x: usize,
    pub z: usize,
}

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

#[derive(Resource, Serialize, Deserialize, Default)]
pub struct Project {
    pub rooms: Vec<Room>,
    pub current_room_idx: usize,
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
           .add_plugins(FrameTimeDiagnosticsPlugin)
           .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
           .add_systems(OnEnter(GameState::Menu), setup_menu)
           .add_systems(Update, (
               menu_system.run_if(in_state(GameState::Menu)),
               hud_update_system.run_if(in_state(GameState::InGame)),
           ))
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
                    MenuAction::StartEditor => next_state.set(GameState::InGame),
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
    mut tile_map: ResMut<TileMap>,
) {
    if !*first_run {
        *first_run = true;
    } else if !project.is_changed() { 
        return; 
    }
    
    if project.rooms.is_empty() { return; }

    // For now, let's stick to full rebuild BUT avoid full world despawn if possible.
    // Actually, real partial update requires tracking WHICH cell changed.
    // Since Project only has is_changed(), we still rebuild the room.
    // BUT we can use the tile_map to despawn only tiles, not other MapEntities.
    for entities in tile_map.entities.values() {
        for &entity in entities {
            commands.entity(entity).despawn_recursive();
        }
    }
    tile_map.entities.clear();

    let room = &project.rooms[project.current_room_idx];
    let mut ramp_cache: std::collections::HashMap<(usize, usize), i32> = std::collections::HashMap::with_capacity(64);

    for x in 0..16 {
        for z in 0..16 {
            let cell = room.cells[x][z];
            if cell.h < 0 { continue; }
            
            let is_even = (x + z) % 2 == 0;

            // Determine the "target terrace height" by following the ramp
            let target_h = *ramp_cache.entry((x, z)).or_insert_with(|| {
                let mut tx = x as i32;
                let mut tz = z as i32;
                let mut steps = 0;
                let mut found_cube_h = None;
                while steps < 16 {
                    let c = room.cells[tx as usize][tz as usize];
                    if matches!(c.tt, TileType::Cube) {
                        found_cube_h = Some(c.h);
                        break;
                    }
                    let (dx, dz) = match c.tt {
                        TileType::WedgeN => (0, -1),
                        TileType::WedgeE => (1, 0),
                        TileType::WedgeS => (0, 1),
                        TileType::WedgeW => (-1, 0),
                        _ => break,
                    };
                    tx += dx;
                    tz += dz;
                    if tx < 0 || tx >= 16 || tz < 0 || tz >= 16 { break; }
                    steps += 1;
                }
                found_cube_h.unwrap_or(cell.h)
            });

            let (mat_top, mat_side) = cache.map.entry((target_h, is_even)).or_insert_with(|| {
                let h_val = (target_h as f32 * 0.1).clamp(0.0, 1.0);
                let base_color = if is_even {
                    Color::srgb(0.4 + h_val * 0.2, 0.7 + h_val * 0.2, 0.2)
                } else {
                    Color::srgb(0.3 + h_val * 0.2, 0.6 + h_val * 0.2, 0.1)
                };
                let side_color = Color::from(LinearRgba::from(base_color) * 0.7);
                (
                    materials.add(StandardMaterial { base_color, ..default() }),
                    materials.add(StandardMaterial { base_color: side_color, ..default() })
                )
            }).clone();

            let (mesh, rot) = match cell.tt {
                TileType::Cube => (assets.cube_mesh.clone(), 0.0),
                TileType::WedgeN => (assets.wedge_mesh.clone(), 0.0),
                TileType::WedgeE => (assets.wedge_mesh.clone(), -std::f32::consts::FRAC_PI_2),
                TileType::WedgeS => (assets.wedge_mesh.clone(), std::f32::consts::PI),
                TileType::WedgeW => (assets.wedge_mesh.clone(), std::f32::consts::FRAC_PI_2),
                _ => (assets.cube_mesh.clone(), 0.0),
            };

            // Spawn top tile (0.5 height unit)
            let h_val = cell.h as f32;
            let mut entities = Vec::new();
            
            let top_id = commands.spawn((PbrBundle {
                mesh, material: mat_top,
                transform: Transform::from_translation(Vec3::new(x as f32, h_val * 0.5 - 0.25, z as f32))
                    .with_scale(Vec3::new(1.0, 0.5, 1.0))
                    .with_rotation(Quat::from_rotation_y(rot)),
                ..default()
            }, TileEntity)).id();
            entities.push(top_id);

            // Spawn single scaled column below
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
}

pub fn hud_update_system(
    diagnostics: Res<DiagnosticsStore>, 
    project: Res<Project>, 
    selection: Option<Res<Selection>>, 
    mut query: Query<&mut Text, With<HudText>>
) {
    let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d: &bevy::diagnostic::Diagnostic| d.smoothed()).unwrap_or(0.0);
    if let Ok(mut text) = query.get_single_mut() {
        if let Some(sel) = selection {
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
