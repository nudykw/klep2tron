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
    mut status_label_query: Query<&mut Text, With<SocketVfxPresetStatusLabel>>,
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

        if let Ok(mut text) = status_label_query.get_single_mut() {
            if let Some(preset_name) = &socket.definition.effect_preset {
                text.sections[0].value = format!("Linked: {}", preset_name);
                text.sections[0].style.color = Color::srgb(0.3, 0.7, 1.0);
            } else {
                text.sections[0].value = "Custom (Unique)".to_string();
                text.sections[0].style.color = Color::srgb(0.5, 0.5, 0.5);
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
    mut vfx_presets: ResMut<VfxPresets>,
    toggle_query: Query<&Interaction, (Changed<Interaction>, With<SocketVfxToggle>)>,
    preset_query: Query<(&Interaction, &SocketVfxPresetItem), Changed<Interaction>>,
    texture_query: Query<(&Interaction, &SocketVfxTextureItem), Changed<Interaction>>,
    group_query: Query<(&Interaction, &SocketVfxGroupItem), Changed<Interaction>>,
    slider_query: Query<(&Slider, &SocketVfxSlider), Changed<Slider>>,
    save_query: Query<&Interaction, (Changed<Interaction>, With<SocketVfxSavePresetButton>)>,
    detach_query: Query<&Interaction, (Changed<Interaction>, With<SocketVfxDetachPresetButton>)>,
    name_input_query: Query<&crate::actor_editor::widgets::TextInput, With<SocketVfxPresetNameInput>>,
    mut toast_events: EventWriter<crate::actor_editor::ToastEvent>,
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
                        socket.definition.effect_preset = Some(preset.0.clone());
                    }
                }
            }
        }
    }

    for interaction in save_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(name_input) = name_input_query.get_single() {
                let name = name_input.value.trim();
                if name.is_empty() {
                    toast_events.send(crate::actor_editor::ToastEvent {
                        message: "Please enter a preset name".to_string(),
                        toast_type: crate::actor_editor::ToastType::Error,
                    });
                    continue;
                }

                // Use the first selected socket as source
                if let Some(&entity) = selected.0.first() {
                    if let Ok(socket) = socket_query.get(entity) {
                        if let Some(effect) = &socket.definition.effect {
                            vfx_presets.library.presets.insert(name.to_string(), effect.clone());
                            
                            crate::actor_editor::vfx_assets::save_vfx_presets(&vfx_presets);
                            
                            toast_events.send(crate::actor_editor::ToastEvent {
                                message: format!("Preset '{}' saved", name),
                                toast_type: crate::actor_editor::ToastType::Success,
                            });
                            
                            // Re-link the current socket to the new preset
                            for &s_entity in selected.0.iter() {
                                if let Ok(mut s_socket) = socket_query.get_mut(s_entity) {
                                    s_socket.definition.effect_preset = Some(name.to_string());
                                }
                            }
                        } else {
                            toast_events.send(crate::actor_editor::ToastEvent {
                                message: "No VFX enabled on this socket to save".to_string(),
                                toast_type: crate::actor_editor::ToastType::Error,
                            });
                        }
                    }
                }
            }
        }
    }

    for interaction in detach_query.iter() {
        if *interaction == Interaction::Pressed {
            for &entity in selected.0.iter() {
                if let Ok(mut socket) = socket_query.get_mut(entity) {
                    socket.definition.effect_preset = None;
                }
            }
            toast_events.send(crate::actor_editor::ToastEvent {
                message: "Detached from preset".to_string(),
                toast_type: crate::actor_editor::ToastType::Info,
            });
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
                if socket.definition.effect.is_some() {
                    socket.definition.effect_preset = None;
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
}
