use bevy::prelude::*;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Local};
use shared::npc::{ActorProject, ActorConfig};
use super::super::{ActorSaveEvent, CurrentProject, SlicingSettings, ActorSocket, ToastEvent, ToastType, EditorStatus, ConfirmationRequestEvent, EditorAction};

pub fn actor_save_system(
    mut save_events: EventReader<ActorSaveEvent>,
    mut current_project: ResMut<CurrentProject>,
    slicing_settings: Res<SlicingSettings>,
    socket_query: Query<&ActorSocket>,
    transform_query: Query<&Transform, With<super::super::Actor3DRoot>>,
    mut status: ResMut<EditorStatus>,
    mut toast_events: EventWriter<ToastEvent>,
    mut modal_events: EventWriter<ConfirmationRequestEvent>,
) {
    for event in save_events.read() {
        let target_name = event.name.as_ref().unwrap_or(&current_project.name);
        
        if target_name.is_empty() {
            toast_events.send(ToastEvent {
                message: "Cannot save: No active project. Import a model first.".to_string(),
                toast_type: ToastType::Error,
            });
            continue;
        }

        // --- CONFLICT CHECK ---
        let project_dir = format!("assets/actors/{}", target_name);
        let actor_file = format!("{}/actor.ron", project_dir);
        
        if !event.force && !current_project.is_saved && Path::new(&actor_file).exists() {
            if let Ok(metadata) = fs::metadata(&actor_file) {
                let created: DateTime<Local> = metadata.created().unwrap_or(metadata.modified().unwrap()).into();
                let created_str = created.format("%Y-%m-%d %H:%M").to_string();
                
                let mut info_str = format!("Existing project info:\n• Created: {}\n", created_str);
                if let Ok(content) = fs::read_to_string(&actor_file) {
                    if let Ok(old_proj) = ron::from_str::<ActorProject>(&content) {
                        info_str.push_str(&format!("• Source Model: {}\n", old_proj.source_path));
                        info_str.push_str(&format!("• Sockets count: {}\n", old_proj.config.sockets.len()));
                    }
                }
                
                modal_events.send(ConfirmationRequestEvent {
                    title: "Project Already Exists".to_string(),
                    message: format!("A project named '{}' already exists on disk.\n\n{}\nDo you want to OVERWRITE it?", target_name, info_str),
                    action: EditorAction::OverwriteProject(target_name.clone()),
                });
                continue;
            }
        }

        if let Some(new_name) = &event.name {
            current_project.name = new_name.clone();
        }

        *status = EditorStatus::Saving;

        let sockets: Vec<_> = socket_query.iter().map(|s| s.definition.clone()).collect();
        
        let project = ActorProject {
            name: current_project.name.clone(),
            source_path: current_project.source_path.clone(),
            cut_top: slicing_settings.top_cut,
            cut_bottom: slicing_settings.bottom_cut,
            scale: transform_query.get_single().map(|t| t.scale).unwrap_or(Vec3::ONE),
            config: ActorConfig { sockets },
        };

        // Save to assets/actors/{name}/actor.ron
        if let Err(e) = fs::create_dir_all(&project_dir) {
            toast_events.send(ToastEvent {
                message: format!("Failed to create project directory: {}", e),
                toast_type: ToastType::Error,
            });
            *status = EditorStatus::Ready;
            continue;
        }

        if let Err(e) = fs::write(&actor_file, ron::ser::to_string_pretty(&project, ron::ser::PrettyConfig::default()).unwrap()) {
            toast_events.send(ToastEvent {
                message: format!("Failed to save actor.ron: {}", e),
                toast_type: ToastType::Error,
            });
        } else {
            current_project.is_saved = true;
            toast_events.send(ToastEvent {
                message: format!("Project '{}' saved successfully", current_project.name),
                toast_type: ToastType::Success,
            });
        }

        *status = EditorStatus::Ready;
    }
}
