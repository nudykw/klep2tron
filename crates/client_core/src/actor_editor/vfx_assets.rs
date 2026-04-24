use bevy::prelude::*;
use shared::npc::VfxPresetLibrary;
use std::fs;

#[derive(Resource, Default)]
pub struct VfxPresets {
    pub library: VfxPresetLibrary,
}

pub fn load_vfx_presets(mut vfx_presets: ResMut<VfxPresets>) {
    let path = "assets/vfx/presets.ron";
    match fs::read_to_string(path) {
        Ok(content) => {
            match ron::from_str::<VfxPresetLibrary>(&content) {
                Ok(library) => {
                    vfx_presets.library = library;
                    info!("VFX Presets loaded: {} items", vfx_presets.library.presets.len());
                }
                Err(e) => {
                    warn!("Failed to parse VFX presets from {}: {}", path, e);
                }
            }
        }
        Err(e) => {
            warn!("VFX presets file not found at {}: {}", path, e);
        }
    }
}
