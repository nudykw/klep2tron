use bevy::prelude::*;
use super::types::*;
use crate::actor_editor::{ActorSocket, vfx_assets::VfxPresets};
use crate::actor_editor::widgets::Slider;

pub fn socket_vfx_ui_sync_system(
    selected: Res<SelectedSocket>,
    socket_query: Query<&ActorSocket>,
    mut toggle_query: Query<&mut BackgroundColor, With<SocketVfxToggle>>,
    mut speed_slider: Query<&mut Slider, (With<SocketVfxSpeedSlider>, Without<SocketVfxScaleSlider>, Without<SocketVfxIntensitySlider>)>,
    mut scale_slider: Query<&mut Slider, (With<SocketVfxScaleSlider>, Without<SocketVfxSpeedSlider>, Without<SocketVfxIntensitySlider>)>,
    mut intensity_slider: Query<&mut Slider, (With<SocketVfxIntensitySlider>, Without<SocketVfxSpeedSlider>, Without<SocketVfxScaleSlider>)>,
) {
    let Some(entity) = selected.0 else { return; };
    let Ok(socket) = socket_query.get(entity) else { return; };
    
    if let Some(effect) = &socket.definition.effect {
        if let Ok(mut bg) = toggle_query.get_single_mut() {
            bg.0 = Color::srgba(0.2, 0.8, 0.2, 0.8); // Green for active
        }
        if let Ok(mut slider) = speed_slider.get_single_mut() { slider.value = effect.speed; }
        if let Ok(mut slider) = scale_slider.get_single_mut() { slider.value = effect.scale; }
        if let Ok(mut slider) = intensity_slider.get_single_mut() { slider.value = effect.intensity; }
    } else {
        if let Ok(mut bg) = toggle_query.get_single_mut() {
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.5); // Black/Dim for inactive
        }
    }
}

pub fn socket_vfx_interaction_system(
    selected: Res<SelectedSocket>,
    mut socket_query: Query<&mut ActorSocket>,
    vfx_presets: Res<VfxPresets>,
    toggle_query: Query<&Interaction, (Changed<Interaction>, With<SocketVfxToggle>)>,
    preset_query: Query<(&Interaction, &SocketVfxPresetItem), Changed<Interaction>>,
    speed_slider: Query<&Slider, (Changed<Slider>, With<SocketVfxSpeedSlider>)>,
    scale_slider: Query<&Slider, (Changed<Slider>, With<SocketVfxScaleSlider>)>,
    intensity_slider: Query<&Slider, (Changed<Slider>, With<SocketVfxIntensitySlider>)>,
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
    }
}
