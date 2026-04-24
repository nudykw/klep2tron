use bevy::prelude::*;
use bevy::color::Srgba;
use shared::npc::EffectType;
use crate::actor_editor::{ActorSocket, ViewportSettings};

pub fn socket_vfx_preview_system(
    mut gizmos: Gizmos,
    time: Res<Time>,
    viewport_settings: Res<ViewportSettings>,
    socket_query: Query<(&GlobalTransform, &ActorSocket)>,
) {
    if !viewport_settings.show_vfx {
        return;
    }

    let t = time.elapsed_seconds();

    for (transform, socket) in socket_query.iter() {
        if let Some(effect) = &socket.definition.effect {
            let pos = transform.translation();
            let rot = transform.compute_transform().rotation;
            
            match effect.effect_type {
                EffectType::Plasma => {
                    // Pulsating cyan sphere/rings
                    let pulse = (t * effect.speed).sin() * 0.2 + 1.0;
                    let radius = 0.2 * effect.scale * pulse;
                    
                    let mut color = Srgba::from(effect.color);
                    color.alpha = 0.8 * effect.intensity;
                    gizmos.sphere(pos, rot, radius, Color::Srgba(color));
                    
                    color.alpha = 0.3 * effect.intensity;
                    gizmos.sphere(pos, rot, radius * 0.7, Color::Srgba(color));
                }
                EffectType::MuzzleFlash => {
                    // Sharp star-like lines, flickering
                    // Burst effect simulated by modulo
                    let burst_t = (t * effect.speed) % 1.0;
                    if burst_t < 0.2 { // Show for first 20% of the cycle
                        let size = 0.4 * effect.scale * (1.0 - burst_t * 2.0).max(0.0);
                        let mut color = Srgba::from(effect.color);
                        color.alpha = (1.0 - burst_t * 5.0).max(0.0) * effect.intensity;
                        
                        gizmos.line(pos - rot * Vec3::X * size, pos + rot * Vec3::X * size, Color::Srgba(color));
                        gizmos.line(pos - rot * Vec3::Y * size, pos + rot * Vec3::Y * size, Color::Srgba(color));
                        gizmos.line(pos - rot * Vec3::Z * size, pos + rot * Vec3::Z * size, Color::Srgba(color));
                        
                        // Diagonal lines for "star" effect
                        let diag = (Vec3::X + Vec3::Y).normalize() * size * 0.7;
                        gizmos.line(pos - rot * diag, pos + rot * diag, Color::Srgba(color));
                    }
                }
                EffectType::Smoke => {
                    // Rising expanding rings
                    let speed = effect.speed * 0.5;
                    for i in 0..4 {
                        let offset = (t * speed + i as f32 * 0.25) % 1.0;
                        let p = pos + rot * Vec3::Y * offset * effect.scale;
                        let r = 0.1 * effect.scale * (1.0 + offset * 3.0);
                        
                        let mut color = Srgba::from(effect.color);
                        color.alpha = (1.0 - offset) * 0.4 * effect.intensity;
                        
                        let normal = Dir3::new(rot * Vec3::Y).unwrap_or(Dir3::Y);
                        gizmos.circle(p, normal, r, Color::Srgba(color));
                    }
                }
            }
        }
    }
}
