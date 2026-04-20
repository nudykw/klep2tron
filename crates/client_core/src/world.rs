use bevy::prelude::*;
use crate::{Project, Room, TileMap};

pub fn setup_game_world(mut commands: Commands, mut project: ResMut<Project>, _asset_server: Res<AssetServer>) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 10000.0,
                ..default()
            },
            transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MapEntity,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });

    let center = Vec3::new(7.5, 0.0, 7.5);
    let radius: f32 = 17.5;
    let angle: f32 = 6.9;
    let height: f32 = 8.0;
    let cam_x = center.x + radius * angle.cos();
    let cam_z = center.z + radius * angle.sin();

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            transform: Transform::from_xyz(cam_x, height, cam_z).looking_at(center, Vec3::Y),
            ..default()
        },
        MapEntity,
    ));

    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(content) = std::fs::read_to_string("assets/map.json") {
        if let Ok(loaded) = serde_json::from_str::<Project>(&content) {
            *project = loaded;
        }
    }
    
    if project.rooms.is_empty() { 
        project.rooms.push(Room::default()); 
    }
}

pub fn cleanup_map(
    mut commands: Commands,
    mut tile_map: ResMut<TileMap>,
    tile_query: Query<Entity, Or<(With<TileEntity>, With<MapEntity>)>>,
) {
    for entity in tile_query.iter() {
        if let Some(ec) = commands.get_entity(entity) {
            ec.despawn_recursive();
        }
    }
    tile_map.entities.clear();
}

#[derive(Component)]
pub struct TileEntity;

#[derive(Component)]
pub struct MapEntity;
