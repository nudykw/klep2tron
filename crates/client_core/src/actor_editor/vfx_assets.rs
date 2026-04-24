use bevy::prelude::*;
use shared::npc::VfxPresetLibrary;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Resource, Default)]
pub struct VfxPresets {
    pub library: VfxPresetLibrary,
}

#[derive(Resource, Default)]
pub struct VfxRegistry {
    pub textures: Vec<(String, Handle<Image>)>,
    pub groups: HashMap<String, Vec<Handle<Image>>>,
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

pub fn register_kenney_textures(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<VfxRegistry>,
) {
    let base_path = "assets/vfx/kenney";
    let dir = Path::new(base_path);
    
    if !dir.exists() {
        warn!("Kenney Particle Pack directory not found at {}", base_path);
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        let mut count = 0;
        let mut temp_groups: HashMap<String, Vec<(String, Handle<Image>)>> = HashMap::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("png") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    let asset_path = format!("vfx/kenney/{}", file_name);
                    let handle = asset_server.load(asset_path);
                    
                    let file_name_str = file_name.to_string();
                    registry.textures.push((file_name_str.clone(), handle.clone()));
                    
                    // Grouping logic: "slash_01.png" -> "slash"
                    if let Some(pos) = file_name.find('_') {
                        let group_name = file_name[..pos].to_string();
                        temp_groups.entry(group_name).or_default().push((file_name_str, handle));
                    }

                    count += 1;
                }
            }
        }
        
        // Finalize groups
        for (group_name, mut items) in temp_groups {
            if items.len() > 1 {
                // Sort items within group by name
                items.sort_by(|a, b| a.0.cmp(&b.0));
                let handles: Vec<Handle<Image>> = items.into_iter().map(|(_, h)| h).collect();
                registry.groups.insert(group_name, handles);
            }
        }

        info!("Kenney textures registered: {} items, {} groups detected", count, registry.groups.len());
        // Sort individual textures for consistent UI
        registry.textures.sort_by(|a, b| a.0.cmp(&b.0));
    }
}
