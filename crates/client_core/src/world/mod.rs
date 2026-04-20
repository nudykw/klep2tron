use bevy::prelude::*;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasBundle;
use crate::{Project, Room, TileMap, GraphicsSettings, QualityLevel, UpscalingMode};
use bevy::pbr::{ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionSettings, ScreenSpaceAmbientOcclusionQualityLevel};
use bevy::render::view::Msaa;
use bevy::core_pipeline::bloom::BloomSettings;

pub mod sky;
pub use sky::*;

pub fn setup_game_world(
    mut commands: Commands, 
    mut project: ResMut<Project>, 
    _asset_server: Res<AssetServer>,
    light_query: Query<Entity, With<DirectionalLight>>,
) {
    if light_query.is_empty() {
        commands.spawn((
            DirectionalLightBundle {
                directional_light: DirectionalLight {
                    shadows_enabled: true,
                    illuminance: 12000.0,
                    ..default()
                },
                transform: Transform::from_xyz(15.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
                cascade_shadow_config: bevy::pbr::CascadeShadowConfigBuilder {
                    first_cascade_far_bound: 20.0,
                    maximum_distance: 200.0,
                    num_cascades: 1, 
                    ..default()
                }.build(),
                ..default()
            },
            MapEntity,
        ));
    }

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
        FogSettings {
            color: Color::srgb(0.05, 0.05, 0.1),
            falloff: FogFalloff::Linear { start: 5.0, end: 25.0 },
            ..default()
        }
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

pub fn apply_graphics_quality_system(
    settings: Res<GraphicsSettings>,
    mut light_query: Query<&mut DirectionalLight>,
    mut fog_query: Query<&mut FogSettings>,
    camera_query: Query<Entity, With<Camera3d>>,
    ssao_query: Query<Entity, With<bevy::pbr::ScreenSpaceAmbientOcclusionSettings>>,
    mut commands: Commands,
    mut initialized: Local<bool>,
    mut msaa: ResMut<Msaa>,
    mut shadow_map: ResMut<bevy::pbr::DirectionalLightShadowMap>,
) {
    let has_cameras = !camera_query.is_empty();
    if !settings.is_changed() && (*initialized && has_cameras) { return; }
    if has_cameras {
        *initialized = true;
        info!("Applying graphics settings: {:?}", *settings);
    }

    // Auto-disable MSAA if SSAO or TAA is used
    if settings.ssao != QualityLevel::Off || settings.upscaling == UpscalingMode::TAA {
        if *msaa != Msaa::Off {
            *msaa = Msaa::Off;
            info!("MSAA disabled for advanced effects");
        }
    } else {
        if *msaa == Msaa::Off {
            *msaa = Msaa::Sample4;
            info!("MSAA re-enabled");
        }
    }
    
    // Shadows
    for mut light in light_query.iter_mut() {
        let enabled = match settings.shadow_quality {
            QualityLevel::Off => false,
            _ => true,
        };
        if light.shadows_enabled != enabled {
            light.shadows_enabled = enabled;
            info!("Directional light shadows set to: {}", enabled);
        }
    }
    if shadow_map.size != settings.shadow_resolution as usize {
        shadow_map.size = settings.shadow_resolution as usize;
        info!("Shadow map resolution changed to {}", shadow_map.size);
    }

    // Fog
    for entity in camera_query.iter() {
        let fog_opt = fog_query.get_mut(entity).ok();
        match settings.fog_quality {
            QualityLevel::Off => {
                if fog_opt.is_some() {
                    commands.entity(entity).remove::<FogSettings>();
                }
            },
            level => {
                let falloff = match level {
                    QualityLevel::Low => FogFalloff::Linear { start: 10.0, end: 40.0 },
                    QualityLevel::Medium => FogFalloff::Linear { start: 5.0, end: 25.0 },
                    _ => FogFalloff::Exponential { density: 0.05 },
                };

                if let Some(mut fog) = fog_opt {
                    fog.falloff = falloff;
                    fog.color = Color::srgb(0.1, 0.1, 0.2);
                } else {
                    info!("Inserting FogSettings into camera");
                    commands.entity(entity).insert(FogSettings {
                        color: Color::srgb(0.1, 0.1, 0.2),
                        falloff,
                        ..default()
                    });
                }
            }
        }
    }
    
    // Post-processing and Upscaling
    for entity in camera_query.iter() {
        // Upscaling & TAA
        match settings.upscaling {
            UpscalingMode::None | UpscalingMode::FSR => {
                commands.entity(entity).remove::<TemporalAntiAliasBundle>();
            },
            UpscalingMode::TAA => {
                commands.entity(entity).insert(TemporalAntiAliasBundle::default());
            },
        }

        // Bloom
        if settings.bloom {
            commands.entity(entity).insert(BloomSettings::default());
        } else {
            commands.entity(entity).remove::<BloomSettings>();
        }

        // SSAO
        let has_ssao = ssao_query.get(entity).is_ok();
        match settings.ssao {
            QualityLevel::Off => {
                if has_ssao {
                    commands.entity(entity).remove::<bevy::pbr::ScreenSpaceAmbientOcclusionBundle>();
                }
            },
            level => {
                let quality = match level {
                    QualityLevel::Low => ScreenSpaceAmbientOcclusionQualityLevel::Low,
                    QualityLevel::Medium => ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                    QualityLevel::High => ScreenSpaceAmbientOcclusionQualityLevel::High,
                    _ => ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
                };

                commands.entity(entity).insert(ScreenSpaceAmbientOcclusionBundle {
                    settings: ScreenSpaceAmbientOcclusionSettings {
                        quality_level: quality,
                    },
                    ..default()
                });
            }
        }
    }
}
