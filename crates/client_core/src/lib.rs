use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// --- Core Data Structures ---

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

pub struct ClientCorePlugin;

impl Plugin for ClientCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
           .init_resource::<Project>()
           .init_resource::<ClientAssets>()
           .add_systems(OnEnter(GameState::Menu), setup_menu)
           .add_systems(Update, menu_system.run_if(in_state(GameState::Menu)))
           .add_systems(OnEnter(GameState::Loading), (cleanup_menu, start_loading))
           .add_systems(Update, check_loading_system.run_if(in_state(GameState::Loading)))
           .add_systems(OnEnter(GameState::InGame), (cleanup_loading, setup_game_world))
           .add_systems(Update, map_rendering_system.run_if(in_state(GameState::InGame)));
    }
}

// --- Systems ---

pub fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), MenuEntity));
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    
    commands.spawn((NodeBundle {
        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        background_color: Color::srgb(0.05, 0.05, 0.1).into(),
        ..default()
    }, MenuEntity)).with_children(|p| {
        p.spawn(TextBundle::from_section("Klep2Tron", TextStyle { font: font.clone(), font_size: 80.0, color: Color::srgb(0.0, 1.0, 1.0) }));
        
        p.spawn(ButtonBundle {
            style: Style { width: Val::Px(250.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::all(Val::Px(40.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
            background_color: Color::srgb(0.1, 0.1, 0.1).into(),
            border_color: Color::srgb(0.3, 0.3, 0.3).into(),
            ..default()
        }).with_children(|p| {
            p.spawn(TextBundle::from_section("START GAME", TextStyle { font: font.clone(), font_size: 24.0, color: Color::WHITE }));
        });
    });
}

pub fn menu_system(mut next_state: ResMut<NextState<GameState>>, query: Query<&Interaction, (Changed<Interaction>, With<Button>)>) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed { next_state.set(GameState::Loading); }
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

pub fn setup_game_world(mut commands: Commands, mut project: ResMut<Project>) {
    // Load map.json
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(content) = std::fs::read_to_string("map.json") {
        if let Ok(loaded) = serde_json::from_str::<Project>(&content) {
            *project = loaded;
        }
    }
    // For wasm, we would need to fetch it, but for now we'll rely on the default or a pre-loaded resource.
    
    if project.rooms.is_empty() { project.rooms.push(Room::default()); }

    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(20.0, 15.0, 20.0).looking_at(Vec3::new(7.5, 0.0, 7.5), Vec3::Y),
        ..default()
    }, MapEntity));

    commands.insert_resource(AmbientLight { color: Color::WHITE, brightness: 500.0 });
}

pub fn map_rendering_system(
    mut commands: Commands,
    project: Res<Project>,
    assets: Res<ClientAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tile_query: Query<Entity, With<TileEntity>>,
) {
    if !project.is_changed() { return; }
    for entity in tile_query.iter() { commands.entity(entity).despawn_recursive(); }

    let room = &project.rooms[project.current_room_idx];
    let top_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.5, 0.8, 0.2), ..default() });
    let side_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.3, 0.5, 0.1), ..default() });

    for x in 0..16 {
        for z in 0..16 {
            let cell = room.cells[x][z];
            if cell.h < 0 { continue; }

            let _pos = Vec3::new(x as f32, 0.0, z as f32);
            
            // Base cube
            if cell.h >= 0 {
                let (mesh, rot) = match cell.tt {
                    TileType::Cube => (assets.cube_mesh.clone(), 0.0),
                    TileType::WedgeN => (assets.wedge_mesh.clone(), 0.0),
                    TileType::WedgeE => (assets.wedge_mesh.clone(), -std::f32::consts::FRAC_PI_2),
                    TileType::WedgeS => (assets.wedge_mesh.clone(), std::f32::consts::PI),
                    TileType::WedgeW => (assets.wedge_mesh.clone(), std::f32::consts::FRAC_PI_2),
                    _ => (assets.cube_mesh.clone(), 0.0),
                };

                commands.spawn((PbrBundle {
                    mesh, material: top_mat.clone(),
                    transform: Transform::from_translation(Vec3::new(x as f32, (cell.h as f32 - 0.5) * 0.5, z as f32))
                        .with_scale(Vec3::new(1.0, 0.5, 1.0))
                        .with_rotation(Quat::from_rotation_y(rot)),
                    ..default()
                }, TileEntity));

                // Walls below
                for y in 0..cell.h {
                    commands.spawn((PbrBundle {
                        mesh: assets.cube_mesh.clone(), material: side_mat.clone(),
                        transform: Transform::from_translation(Vec3::new(x as f32, (y as f32 - 0.5) * 0.5, z as f32))
                            .with_scale(Vec3::new(1.0, 0.5, 1.0)),
                        ..default()
                    }, TileEntity));
                }
            }
        }
    }
}
