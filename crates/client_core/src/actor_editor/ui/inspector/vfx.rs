use bevy::prelude::*;
use super::types::*;
use crate::actor_editor::{ActorSocket, vfx_assets::VfxPresets};
use crate::actor_editor::widgets::Slider;

pub fn socket_vfx_ui_sync_system(
    selected: Res<SelectedSocket>,
    socket_query: Query<&ActorSocket>,
    mut toggle_query: Query<&mut BackgroundColor, With<SocketVfxToggle>>,
    mut slider_query: Query<(&mut Slider, &Interaction, &SocketVfxSlider)>,
    mut texture_items_query: Query<(&SocketVfxTextureItem, &mut BackgroundColor), Without<SocketVfxToggle>>,
    mut group_items_query: Query<(&SocketVfxGroupItem, &mut BackgroundColor), (Without<SocketVfxToggle>, Without<SocketVfxTextureItem>)>,
) {
    let Some(&entity) = selected.0.first() else { return; };
    let Ok(socket) = socket_query.get(entity) else { return; };
    
    if let Some(effect) = &socket.definition.effect {
        if let Ok(mut bg) = toggle_query.get_single_mut() {
            bg.0 = Color::srgba(0.2, 0.8, 0.2, 0.8);
        }
        
        for (mut slider, interaction, marker) in slider_query.iter_mut() {
            if *interaction != Interaction::None { continue; }
            match marker {
                SocketVfxSlider::EmissionRate => slider.value = effect.emission.rate,
                SocketVfxSlider::EmissionLifetime => slider.value = effect.emission.lifetime,
                SocketVfxSlider::EmissionJitter => slider.value = effect.emission.jitter,
                SocketVfxSlider::MotionSpeed => slider.value = effect.motion.speed,
                SocketVfxSlider::MotionSpread => slider.value = effect.motion.spread,
                SocketVfxSlider::MotionGravity => slider.value = effect.motion.gravity,
                SocketVfxSlider::MotionDrag => slider.value = effect.motion.drag,
                SocketVfxSlider::VisualsScale => slider.value = effect.visuals.scale,
                SocketVfxSlider::VisualsSizeStart => slider.value = effect.visuals.size_start,
                SocketVfxSlider::VisualsSizeEnd => slider.value = effect.visuals.size_end,
            }
        }
        
        // Highlight selected texture or group
        for (item, mut bg) in texture_items_query.iter_mut() {
            if effect.asset_path.as_ref() == Some(&item.0) {
                *bg = Color::srgba(0.0, 0.6, 1.0, 0.4).into();
            } else {
                *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
            }
        }
        for (item, mut bg) in group_items_query.iter_mut() {
            if effect.asset_path.as_ref() == Some(&item.0) {
                *bg = Color::srgba(0.0, 1.0, 0.6, 0.4).into(); // Teal for groups
            } else {
                *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
            }
        }
    } else {
        if let Ok(mut bg) = toggle_query.get_single_mut() {
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.5); // Black/Dim for inactive
        }
        for (_, mut bg) in texture_items_query.iter_mut() {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
        }
        for (_, mut bg) in group_items_query.iter_mut() {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
        }
    }
}

pub fn socket_vfx_interaction_system(
    selected: Res<SelectedSocket>,
    mut socket_query: Query<&mut ActorSocket>,
    vfx_presets: Res<VfxPresets>,
    toggle_query: Query<&Interaction, (Changed<Interaction>, With<SocketVfxToggle>)>,
    preset_query: Query<(&Interaction, &SocketVfxPresetItem), Changed<Interaction>>,
    texture_query: Query<(&Interaction, &SocketVfxTextureItem), Changed<Interaction>>,
    group_query: Query<(&Interaction, &SocketVfxGroupItem), Changed<Interaction>>,
    slider_query: Query<(&Slider, &SocketVfxSlider), Changed<Slider>>,
) {
    if selected.0.is_empty() { return; }
    
    // 1. Handle non-slider interactions (Toggle, Presets, Assets)
    // We apply these once per click
    for interaction in toggle_query.iter() {
        if *interaction == Interaction::Pressed {
            for &entity in selected.0.iter() {
                if let Ok(mut socket) = socket_query.get_mut(entity) {
                    if socket.definition.effect.is_some() {
                        socket.definition.effect = None;
                    } else {
                        socket.definition.effect = Some(Default::default());
                    }
                }
            }
        }
    }
    
    for (interaction, preset) in preset_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(config) = vfx_presets.library.presets.get(&preset.0) {
                for &entity in selected.0.iter() {
                    if let Ok(mut socket) = socket_query.get_mut(entity) {
                        socket.definition.effect = Some(config.clone());
                    }
                }
            }
        }
    }

    for (interaction, texture) in texture_query.iter() {
        if *interaction == Interaction::Pressed {
            for &entity in selected.0.iter() {
                if let Ok(mut socket) = socket_query.get_mut(entity) {
                    if let Some(effect) = &mut socket.definition.effect {
                        effect.asset_path = Some(texture.0.clone());
                        effect.effect_type = shared::npc::EffectType::Hanabi;
                    }
                }
            }
        }
    }
    
    for (interaction, group) in group_query.iter() {
        if *interaction == Interaction::Pressed {
            for &entity in selected.0.iter() {
                if let Ok(mut socket) = socket_query.get_mut(entity) {
                    if let Some(effect) = &mut socket.definition.effect {
                        effect.asset_path = Some(group.0.clone());
                        effect.effect_type = shared::npc::EffectType::Hanabi;
                    }
                }
            }
        }
    }

    // 2. Handle slider changes
    for (slider, marker) in slider_query.iter() {
        for &entity in selected.0.iter() {
            if let Ok(mut socket) = socket_query.get_mut(entity) {
                if let Some(effect) = &mut socket.definition.effect {
                    match marker {
                        SocketVfxSlider::EmissionRate => effect.emission.rate = slider.value,
                        SocketVfxSlider::EmissionLifetime => effect.emission.lifetime = slider.value,
                        SocketVfxSlider::EmissionJitter => effect.emission.jitter = slider.value,
                        SocketVfxSlider::MotionSpeed => effect.motion.speed = slider.value,
                        SocketVfxSlider::MotionSpread => effect.motion.spread = slider.value,
                        SocketVfxSlider::MotionGravity => effect.motion.gravity = slider.value,
                        SocketVfxSlider::MotionDrag => effect.motion.drag = slider.value,
                        SocketVfxSlider::VisualsScale => effect.visuals.scale = slider.value,
                        SocketVfxSlider::VisualsSizeStart => effect.visuals.size_start = slider.value,
                        SocketVfxSlider::VisualsSizeEnd => effect.visuals.size_end = slider.value,
                    }
                }
            }
        }
    }
}
