use bevy::prelude::*;
use super::types::*;
use crate::actor_editor::{ActorSocket, vfx_assets::VfxPresets};
use crate::actor_editor::widgets::Slider;

pub fn socket_vfx_ui_sync_system(
    selected: Res<SelectedSocket>,
    socket_query: Query<&ActorSocket>,
    mut toggle_query: Query<&mut BackgroundColor, With<SocketVfxToggle>>,
    mut speed_slider: Query<(&mut Slider, &Interaction), (With<SocketVfxSpeedSlider>, Without<SocketVfxScaleSlider>, Without<SocketVfxIntensitySlider>, Without<SocketVfxLifetimeSlider>)>,
    mut scale_slider: Query<(&mut Slider, &Interaction), (With<SocketVfxScaleSlider>, Without<SocketVfxSpeedSlider>, Without<SocketVfxIntensitySlider>, Without<SocketVfxLifetimeSlider>)>,
    mut intensity_slider: Query<(&mut Slider, &Interaction), (With<SocketVfxIntensitySlider>, Without<SocketVfxSpeedSlider>, Without<SocketVfxScaleSlider>, Without<SocketVfxLifetimeSlider>)>,
    mut lifetime_slider: Query<(&mut Slider, &Interaction), (With<SocketVfxLifetimeSlider>, Without<SocketVfxSpeedSlider>, Without<SocketVfxScaleSlider>, Without<SocketVfxIntensitySlider>)>,
    mut texture_items_query: Query<(&SocketVfxTextureItem, &mut BackgroundColor), Without<SocketVfxToggle>>,
    mut group_items_query: Query<(&SocketVfxGroupItem, &mut BackgroundColor), (Without<SocketVfxToggle>, Without<SocketVfxTextureItem>)>,
) {
    let Some(entity) = selected.0 else { return; };
    let Ok(socket) = socket_query.get(entity) else { return; };
    
    if let Some(effect) = &socket.definition.effect {
        if let Ok(mut bg) = toggle_query.get_single_mut() {
            bg.0 = Color::srgba(0.2, 0.8, 0.2, 0.8); // Green for active
        }
        if let Ok((mut slider, interaction)) = speed_slider.get_single_mut() { 
            if *interaction == Interaction::None { slider.value = effect.speed; }
        }
        if let Ok((mut slider, interaction)) = scale_slider.get_single_mut() { 
            if *interaction == Interaction::None { slider.value = effect.scale; }
        }
        if let Ok((mut slider, interaction)) = intensity_slider.get_single_mut() { 
            if *interaction == Interaction::None { slider.value = effect.intensity; }
        }
        if let Ok((mut slider, interaction)) = lifetime_slider.get_single_mut() { 
            if *interaction == Interaction::None { slider.value = effect.lifetime; }
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
    speed_slider: Query<&Slider, (Changed<Slider>, With<SocketVfxSpeedSlider>)>,
    scale_slider: Query<&Slider, (Changed<Slider>, With<SocketVfxScaleSlider>)>,
    intensity_slider: Query<&Slider, (Changed<Slider>, With<SocketVfxIntensitySlider>)>,
    lifetime_slider: Query<&Slider, (Changed<Slider>, With<SocketVfxLifetimeSlider>)>,
) {
    let Some(entity) = selected.0 else { return; };
    let Ok(mut socket) = socket_query.get_mut(entity) else { return; };
    
    // Toggle
    for interaction in toggle_query.iter() {
        if *interaction == Interaction::Pressed {
            if socket.definition.effect.is_some() {
                socket.definition.effect = None;
            } else {
                socket.definition.effect = Some(Default::default());
            }
        }
    }
    
    // Presets
    for (interaction, preset) in preset_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(config) = vfx_presets.library.presets.get(&preset.0) {
                socket.definition.effect = Some(config.clone());
            }
        }
    }

    // Textures
    for (interaction, texture) in texture_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(effect) = &mut socket.definition.effect {
                effect.asset_path = Some(texture.0.clone());
                effect.effect_type = shared::npc::EffectType::Hanabi;
            }
        }
    }

    // Groups
    for (interaction, group) in group_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(effect) = &mut socket.definition.effect {
                effect.asset_path = Some(group.0.clone());
                effect.effect_type = shared::npc::EffectType::Hanabi;
            }
        }
    }
    
    // Sliders (only if effect is active)
    if let Some(effect) = &mut socket.definition.effect {
        if let Ok(slider) = speed_slider.get_single() { 
            if effect.speed != slider.value { effect.speed = slider.value; }
        }
        if let Ok(slider) = scale_slider.get_single() { 
            if effect.scale != slider.value { effect.scale = slider.value; }
        }
        if let Ok(slider) = intensity_slider.get_single() { 
            if effect.intensity != slider.value { effect.intensity = slider.value; }
        }
        if let Ok(slider) = lifetime_slider.get_single() { 
            if effect.lifetime != slider.value { effect.lifetime = slider.value; }
        }
    }
}
