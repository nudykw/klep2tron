use bevy::prelude::*;
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
                    // Multi-layered High-Energy Plasma
                    let pulse = (t * effect.speed).sin() * 0.1 + 1.0;
                    let base_radius = 0.2 * effect.scale * pulse;
                    
                    // Use HSLA for better color shifting
                    let mut hsla = Hsla::from(effect.color);
                    hsla.alpha = 0.9 * effect.intensity;
                    
                    // 1. Bright Core
                    gizmos.sphere(pos, rot, base_radius * 0.4, Color::Hsla(hsla));
                    
                    // 2. Middle Layer (Shift Hue towards Blue)
                    let mut mid_hsla = hsla;
                    mid_hsla.hue = (mid_hsla.hue + 40.0) % 360.0; // Shift towards blue
                    mid_hsla.alpha = 0.4 * effect.intensity;
                    gizmos.sphere(pos, rot, base_radius * 0.8, Color::Hsla(mid_hsla));
                    
                    // 3. Outer Layer (Shift Hue towards Purple/Magenta)
                    let mut outer_hsla = hsla;
                    outer_hsla.hue = (outer_hsla.hue + 80.0) % 360.0; // Shift towards purple
                    outer_hsla.alpha = 0.15 * effect.intensity;
                    gizmos.sphere(pos, rot, base_radius * 1.2, Color::Hsla(outer_hsla));
                    
                    // 4. Rotating Energy Rings
                    for i in 0..2 {
                        let ring_rot = Quat::from_axis_angle(Vec3::Y, t * effect.speed * (1.0 + i as f32)) * 
                                       Quat::from_axis_angle(Vec3::X, t * 0.5);
                        let ring_color = if i == 0 { mid_hsla } else { outer_hsla };
                        gizmos.circle(pos, Dir3::new(ring_rot * Vec3::Z).unwrap_or(Dir3::Z), base_radius * (1.1 + i as f32 * 0.2), Color::Hsla(ring_color));
                    }
                }
                EffectType::MuzzleFlash => {
                    // Explosive Star Burst
                    let cycle_speed = effect.speed * 2.5; 
                    let burst_t = (t * cycle_speed) % 1.0;
                    
                    let active_window = 0.12; 
                    if burst_t < active_window {
                        let normalized_t = burst_t / active_window;
                        let size = 0.6 * effect.scale * (1.0 - normalized_t).powf(0.3);
                        
                        let mut hsla = Hsla::from(effect.color);
                        // Shift from core color to orange-red
                        hsla.hue = (hsla.hue - normalized_t * 30.0).max(0.0);
                        hsla.alpha = (1.0 - normalized_t).max(0.0) * effect.intensity;
                        
                        let c = Color::Hsla(hsla);
                        
                        // 8-point star logic
                        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                            gizmos.line(pos - rot * axis * size, pos + rot * axis * size, c);
                        }
                        
                        // Diagonal bursts
                        let diags = [
                            (Vec3::X + Vec3::Y).normalize(),
                            (Vec3::X - Vec3::Y).normalize(),
                            (Vec3::Y + Vec3::Z).normalize(),
                            (Vec3::Y - Vec3::Z).normalize(),
                        ];
                        for d in diags {
                            let world_d = rot * d;
                            gizmos.line(pos - world_d * size * 0.8, pos + world_d * size * 0.8, c);
                        }
                        
                        // Central flash sphere
                        let mut flash_hsla = hsla;
                        flash_hsla.lightness = (flash_hsla.lightness + 0.3).min(1.0);
                        gizmos.sphere(pos, rot, size * 0.2, Color::Hsla(flash_hsla));
                    }
                }
                EffectType::Smoke => {
                    // Drifting expanding smoke particles
                    let speed = effect.speed * 0.3;
                    for i in 0..6 {
                        let offset = (t * speed + i as f32 * 0.16) % 1.0;
                        
                        // Add some horizontal "drift" using sine waves
                        let drift_x = (t + i as f32).sin() * 0.2 * offset;
                        let drift_z = (t * 0.8 + i as f32).cos() * 0.2 * offset;
                        
                        let p = pos + rot * (Vec3::Y * offset * 1.5 + Vec3::new(drift_x, 0.0, drift_z)) * effect.scale;
                        let r = 0.1 * effect.scale * (1.0 + offset * 5.0);
                        
                        let mut hsla = Hsla::from(effect.color);
                        hsla.alpha = (1.0 - offset).powf(1.8) * 0.6 * effect.intensity;
                        
                        // Draw circles with slight rotation for "volume" look
                        let tilt = Quat::from_axis_angle(Vec3::X, offset * 2.0);
                        let normal = Dir3::new(rot * tilt * Vec3::Y).unwrap_or(Dir3::Y);
                        gizmos.circle(p, normal, r, Color::Hsla(hsla));
                        
                        // Add a smaller inner circle for depth
                        hsla.alpha *= 0.5;
                        gizmos.circle(p, normal, r * 0.7, Color::Hsla(hsla));
                    }
                }
                EffectType::Hanabi => {}
            }
        }
    }
}
