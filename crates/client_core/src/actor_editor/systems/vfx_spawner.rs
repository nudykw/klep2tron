use bevy::prelude::*;
use bevy_hanabi::*;
use shared::npc::EffectType;
use crate::actor_editor::ActorSocket;

#[derive(Component)]
pub struct SocketVfxInstance {
    pub socket_entity: Entity,
}

pub fn socket_vfx_spawner_system(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    socket_query: Query<(Entity, &GlobalTransform, &ActorSocket), Changed<ActorSocket>>,
    instance_query: Query<(Entity, &SocketVfxInstance)>,
) {
    for (socket_entity, transform, socket) in socket_query.iter() {
        // Despawn old instances for this socket
        for (instance_entity, instance) in instance_query.iter() {
            if instance.socket_entity == socket_entity {
                commands.entity(instance_entity).despawn_recursive();
            }
        }

        // Spawn new instance if Hanabi effect is selected
        if let Some(effect_config) = &socket.definition.effect {
            if effect_config.effect_type == EffectType::Hanabi {
                // For now, create a simple procedural Hanabi effect as a placeholder
                // In a real implementation, we would fetch from VfxRegistry
                
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec4::new(1.0, 1.0, 0.0, 1.0)); // Yellow
                gradient.add_key(1.0, Vec4::new(1.0, 0.0, 0.0, 0.0)); // Red transparent

                let writer = ExprWriter::new();

                // Give some initial velocity
                let init_pos = SetPositionSphereModifier {
                    center: writer.lit(Vec3::ZERO).expr(),
                    radius: writer.lit(0.05).expr(),
                    dimension: ShapeDimension::Volume,
                };
                let init_vel = SetVelocitySphereModifier {
                    center: writer.lit(Vec3::ZERO).expr(),
                    speed: writer.lit(1.0).expr(),
                };
                
                // Lifetime
                let lifetime = writer.lit(0.5).expr();
                let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

                let effect = EffectAsset::new(32768, Spawner::rate(100.0.into()), writer.finish())
                    .with_name("KenneyPlaceholder")
                    .init(init_pos)
                    .init(init_vel)
                    .init(init_lifetime)
                    .render(ColorOverLifetimeModifier { gradient });

                let effect_handle = effects.add(effect);

                commands.spawn((
                    ParticleEffectBundle {
                        effect: ParticleEffect::new(effect_handle),
                        transform: transform.compute_transform(),
                        ..default()
                    },
                    SocketVfxInstance {
                        socket_entity,
                    },
                ));
            }
        }
    }
}
