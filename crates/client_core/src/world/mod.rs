use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::post_process::bloom::Bloom;
use crate::{Project, Room, TileMap, GraphicsSettings, QualityLevel, UpscalingMode};
use bevy::pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel};
use bevy::render::view::Msaa;

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
            DirectionalLight {
                shadow_maps_enabled: true,
                illuminance: 12000.0,
                ..default()
            },
            Transform::from_xyz(15.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
            CascadeShadowConfigBuilder {
                first_cascade_far_bound: 20.0,
                maximum_distance: 200.0,
                num_cascades: 1,
                ..default()
            }
            .build(),
            MapEntity,
        ));
    }

    let center = Vec3::new(7.5, 0.0, 7.5);
    let radius: f32 = 17.5;
    let angle: f32 = 6.9;
    let height: f32 = 8.0;
    let cam_x = center.x + radius * angle.cos();
    let cam_z = center.z + radius * angle.sin();

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        Transform::from_xyz(cam_x, height, cam_z).looking_at(center, Vec3::Y),
        AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            affects_lightmapped_meshes: false,
        },
        DistanceFog {
            color: Color::srgb(0.05, 0.05, 0.1),
            falloff: FogFalloff::Linear { start: 5.0, end: 25.0 },
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
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
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
    mut camera_query: Query<(Entity, Option<&Msaa>), With<Camera3d>>,
    fog_query: Query<Entity, With<DistanceFog>>,
    ssao_query: Query<Entity, With<ScreenSpaceAmbientOcclusion>>,
    mut commands: Commands,
    mut initialized: Local<bool>,
    mut shadow_map: ResMut<DirectionalLightShadowMap>,
    adapter: Option<Res<RenderAdapterInfo>>,
) {
    let has_cameras = !camera_query.is_empty();
    if settings.is_loading { return; }
    if !settings.is_changed() && (*initialized && has_cameras) { return; }
    if has_cameras {
        *initialized = true;
        let gpu_name = adapter.map(|a| a.name.clone()).unwrap_or_else(|| "Unknown".to_string());
        info!("Applying graphics settings: {:?} (Current GPU: {})", *settings, gpu_name);
    }

    // Auto-disable MSAA if SSAO or TAA is used. MSAA is a per-camera component in 0.19.
    let want_msaa = if settings.ssao != QualityLevel::Off || settings.upscaling == UpscalingMode::TAA {
        Msaa::Off
    } else {
        Msaa::Sample4
    };
    for (entity, current) in camera_query.iter() {
        if current != Some(&want_msaa) {
            commands.entity(entity).insert(want_msaa);
            if want_msaa == Msaa::Off {
                info!("MSAA disabled for advanced effects");
            } else {
                info!("MSAA re-enabled");
            }
        }
    }

    // Shadows
    for mut light in light_query.iter_mut() {
        let enabled = match settings.shadow_quality {
            QualityLevel::Off => false,
            _ => true,
        };
        if light.shadow_maps_enabled != enabled {
            light.shadow_maps_enabled = enabled;
            info!("Directional light shadows set to: {}", enabled);
        }
    }
    if shadow_map.size != settings.shadow_resolution as usize {
        shadow_map.size = settings.shadow_resolution as usize;
        info!("Shadow map resolution changed to {}", shadow_map.size);
    }

    // Fog (per-camera component)
    for (entity, _) in camera_query.iter() {
        let has_fog = fog_query.contains(entity);
        match settings.fog_quality {
            QualityLevel::Off => {
                if has_fog {
                    commands.entity(entity).remove::<DistanceFog>();
                }
            },
            level => {
                let falloff = match level {
                    QualityLevel::Low => FogFalloff::Linear { start: 10.0, end: 40.0 },
                    QualityLevel::Medium => FogFalloff::Linear { start: 5.0, end: 25.0 },
                    _ => FogFalloff::Exponential { density: 0.05 },
                };

                if has_fog {
                    // Fog already exists on this camera; overwrite the values.
                    commands.entity(entity).insert(DistanceFog {
                        color: Color::srgb(0.1, 0.1, 0.2),
                        falloff,
                        ..default()
                    });
                } else {
                    info!("Inserting DistanceFog into camera");
                    commands.entity(entity).insert(DistanceFog {
                        color: Color::srgb(0.1, 0.1, 0.2),
                        falloff,
                        ..default()
                    });
                }
            }
        }
    }

    // Post-processing and Upscaling
    for (entity, _) in camera_query.iter() {
        // Upscaling & TAA
        match settings.upscaling {
            UpscalingMode::None | UpscalingMode::FSR => {
                commands.entity(entity).remove::<TemporalAntiAliasing>();
            },
            UpscalingMode::TAA => {
                commands.entity(entity).insert(TemporalAntiAliasing::default());
            },
        }

        // Bloom
        if settings.bloom {
            commands.entity(entity).insert(Bloom::default());
        } else {
            commands.entity(entity).remove::<Bloom>();
        }

        // SSAO
        let has_ssao = ssao_query.contains(entity);
        match settings.ssao {
            QualityLevel::Off => {
                if has_ssao {
                    commands.entity(entity).remove::<ScreenSpaceAmbientOcclusion>();
                }
            },
            level => {
                let quality = match level {
                    QualityLevel::Low => ScreenSpaceAmbientOcclusionQualityLevel::Low,
                    QualityLevel::Medium => ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                    QualityLevel::High => ScreenSpaceAmbientOcclusionQualityLevel::High,
                    _ => ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
                };

                commands.entity(entity).insert(ScreenSpaceAmbientOcclusion {
                    quality_level: quality,
                    ..default()
                });
            }
        }
    }
}
