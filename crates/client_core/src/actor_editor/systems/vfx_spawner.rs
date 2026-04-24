use bevy::prelude::*;
use bevy_hanabi::*;
use crate::actor_editor::{ActorSocket, vfx_assets::VfxRegistry};

#[derive(Component)]
pub struct SocketVfxInstance {
    pub socket_entity: Entity,
}

#[derive(Component, PartialEq, Clone)]
pub struct ActiveVfxConfig(pub Option<shared::npc::EffectConfig>);

pub fn socket_vfx_sync_system(
    mut vfx_query: Query<(Entity, &mut Transform, &SocketVfxInstance)>,
    socket_query: Query<(&GlobalTransform, &ActorSocket)>,
    mut commands: Commands,
) {
    for (entity, mut transform, instance) in vfx_query.iter_mut() {
        if let Ok((socket_transform, socket)) = socket_query.get(instance.socket_entity) {
            let (_, rotation, translation) = socket_transform.to_scale_rotation_translation();
            transform.translation = translation;
            transform.rotation = rotation;
            
            if let Some(effect_config) = &socket.definition.effect {
                transform.scale = Vec3::splat(effect_config.scale);
            }
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn socket_vfx_spawner_system(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    _vfx_registry: Res<VfxRegistry>,
    socket_query: Query<(Entity, &GlobalTransform, &ActorSocket), Changed<ActorSocket>>,
    instance_query: Query<(Entity, &SocketVfxInstance)>,
    active_config_query: Query<&ActiveVfxConfig>,
) {
    for (socket_entity, _transform, socket) in socket_query.iter() {
        let new_config = &socket.definition.effect;
        
        let mut needs_recreate = true;
        if let Ok(active) = active_config_query.get(socket_entity) {
            if let (Some(old), Some(new)) = (&active.0, new_config) {
                // Only recreate if TYPE, COLOR, or LIFETIME changed significantly
                // Scale/Speed/Intensity updates are handled by live sync or doesn't need asset recreation
                if old.effect_type == new.effect_type && old.asset_path == new.asset_path && old.lifetime == new.lifetime {
                    needs_recreate = false;
                }
            } else if active.0.is_none() && new_config.is_none() {
                needs_recreate = false;
            }
        }
        
        if !needs_recreate {
            continue;
        }
        
        commands.entity(socket_entity).insert(ActiveVfxConfig(new_config.clone()));

        for (instance_entity, instance) in instance_query.iter() {
            if instance.socket_entity == socket_entity {
                commands.entity(instance_entity).despawn_recursive();
            }
        }

        if let Some(effect_config) = new_config {
            let writer = ExprWriter::new();

            // Refined Procedural Parameters
            let (base_rate, base_lifetime, base_size, base_speed) = match effect_config.effect_type {
                shared::npc::EffectType::Hanabi => (300.0, 1.2, 0.05, 2.0),
                shared::npc::EffectType::Smoke => (60.0, 2.5, 0.08, 0.4), 
                shared::npc::EffectType::Plasma => (500.0, 0.6, 0.02, 6.0),
                shared::npc::EffectType::MuzzleFlash => (1000.0, 0.15, 0.1, 4.0),
            };

            let rate = base_rate * effect_config.intensity;
            let spawner = Spawner::rate(rate.into());

            let init_pos = SetPositionSphereModifier {
                center: writer.lit(Vec3::ZERO).expr(),
                radius: writer.lit(0.02).expr(),
                dimension: ShapeDimension::Volume,
            };
            
            let init_vel = SetVelocitySphereModifier {
                center: writer.lit(Vec3::ZERO).expr(),
                speed: writer.lit(base_speed * effect_config.speed).expr(),
            };
            
            let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(base_lifetime * effect_config.lifetime).expr());

            let color = effect_config.color.to_srgba();
            let mut gradient = Gradient::new();
            gradient.add_key(0.0, Vec4::new(color.red, color.green, color.blue, color.alpha));
            gradient.add_key(0.7, Vec4::new(color.red, color.green, color.blue, color.alpha));
            gradient.add_key(1.0, Vec4::new(color.red, color.green, color.blue, 0.0));
            
            let render_color = ColorOverLifetimeModifier { gradient };
            let init_size = SetAttributeModifier::new(Attribute::SIZE, writer.lit(base_size).expr());

            let effect_asset = EffectAsset::new(4096, spawner, writer.finish())
                .with_name("Procedural_VFX")
                .with_simulation_space(SimulationSpace::Local)
                .init(init_pos)
                .init(init_vel)
                .init(init_lifetime)
                .init(init_size)
                .render(render_color);

            let effect_handle = effects.add(effect_asset);

            commands.spawn((
                ParticleEffectBundle {
                    effect: ParticleEffect::new(effect_handle),
                    ..default()
                },
                SocketVfxInstance {
                    socket_entity,
                },
                Name::new("VFX_Procedural"),
            ));
        }
    }
}
