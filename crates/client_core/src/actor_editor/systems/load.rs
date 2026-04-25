use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use shared::npc::ActorProject;
use super::super::{ActorLoadEvent, ActorImportEvent, CurrentProject, SlicingSettings, PendingSockets, ToastEvent, ToastType, EditorStatus};

pub fn actor_load_system(
    mut load_events: EventReader<ActorLoadEvent>,
    mut import_events: EventWriter<ActorImportEvent>,
    mut current_project: ResMut<CurrentProject>,
    mut slicing_settings: ResMut<SlicingSettings>,
    mut pending_sockets: ResMut<PendingSockets>,
    mut pending_import: ResMut<super::super::PendingImport>,
    mut status: ResMut<EditorStatus>,
    mut toast_events: EventWriter<ToastEvent>,
    mut opt_settings: ResMut<crate::actor_editor::systems::optimization::OptimizationSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pending_slices: ResMut<super::super::PendingSlices>,
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
        slicing_settings.rim_thickness = project.rim_thickness;
        slicing_settings.last_top = project.cut_top;
        slicing_settings.last_bottom = project.cut_bottom;
        slicing_settings.trigger_slice = false; 
        slicing_settings.suppress_undo = true; // Don't record the load as an undoable action
        
        // 3. Restore Optimization Settings
        if let Some(budget) = project.optimization_budget {
            opt_settings.target_triangles = budget;
            opt_settings.is_optimized = true;
        } else {
            opt_settings.is_optimized = false;
        }

        // 4. Load Pre-sliced Meshes if they exist
        pending_slices.0.clear();
        let project_dir = path.parent().unwrap_or(Path::new("."));
        
        let mut load_part = |part: shared::npc::ActorPart, filename_opt: &Option<String>| {
            if let Some(filename) = filename_opt {
                let mesh_file_path = project_dir.join(filename);
                if mesh_file_path.exists() {
                    match super::export::import_mesh_from_k2m(&mesh_file_path) {
                        Ok(mesh) => {
                            let handle = meshes.add(mesh);
                            pending_slices.0.insert(part, handle);
                        }
                        Err(e) => warn!("Failed to load pre-sliced mesh {}: {}", filename, e),
                    }
                }
            }
        };

        load_part(shared::npc::ActorPart::Head, &project.head_mesh);
        load_part(shared::npc::ActorPart::Body, &project.body_mesh);
        load_part(shared::npc::ActorPart::Engine, &project.legs_mesh);

        // 5. Prepare Sockets and Scale for spawning
        pending_sockets.0 = project.config.sockets;
        pending_import.scale = Some(project.scale);

        // 6. Trigger Model Import
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
