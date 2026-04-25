use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;
use shared::npc::ActorProject;
use super::super::{ActorLoadEvent, ActorImportEvent, CurrentProject, SlicingSettings, PendingSockets, ToastEvent, ToastType, EditorStatus};

pub fn actor_load_system(
    mut load_events: EventReader<ActorLoadEvent>,
    mut import_events: EventWriter<ActorImportEvent>,
    mut current_project: ResMut<CurrentProject>,
    mut slicing_settings: ResMut<SlicingSettings>,
    mut pending_sockets: ResMut<PendingSockets>,
    mut status: ResMut<EditorStatus>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in load_events.read() {
        let path = &event.0;
        
        info!("Loading project from: {:?}", path);
        
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                toast_events.send(ToastEvent {
                    message: format!("Failed to read project file: {}", e),
                    toast_type: ToastType::Error,
                });
                continue;
            }
        };

        let project: ActorProject = match ron::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                toast_events.send(ToastEvent {
                    message: format!("Failed to parse project file: {}", e),
                    toast_type: ToastType::Error,
                });
                continue;
            }
        };

        // 1. Update Project Metadata
        current_project.name = project.name;
        current_project.source_path = project.source_path.clone();
        current_project.is_saved = true;

        // 2. Restore Slicing Settings
        slicing_settings.top_cut = project.cut_top;
        slicing_settings.bottom_cut = project.cut_bottom;
        slicing_settings.last_top = project.cut_top;
        slicing_settings.last_bottom = project.cut_bottom;
        slicing_settings.trigger_slice = false; 
        slicing_settings.suppress_undo = true; // Don't record the load as an undoable action

        // 3. Prepare Sockets for spawning
        pending_sockets.0 = project.config.sockets;

        // 4. Trigger Model Import
        // Path should be relative to assets in project, but we need absolute for the event
        let source_path = PathBuf::from(&project.source_path);
        
        // Check if model exists
        let current_dir = std::env::current_dir().unwrap_or_default();
        let full_path = current_dir.join("assets").join(&source_path);
        
        if !full_path.exists() {
             toast_events.send(ToastEvent {
                message: format!("Source model not found: assets/{}", project.source_path),
                toast_type: ToastType::Error,
            });
            // We still set metadata, but import will fail
        }

        import_events.send(ActorImportEvent(full_path, false));
        
        toast_events.send(ToastEvent {
            message: format!("Project '{}' loaded. Importing model...", current_project.name),
            toast_type: ToastType::Info,
        });
        
        *status = EditorStatus::Loading;
    }
}
