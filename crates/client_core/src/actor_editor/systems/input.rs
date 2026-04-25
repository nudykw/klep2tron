use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use rfd::FileDialog;
use super::super::{ActorImportEvent, ActorLoadEvent, SlicingSettings, ToastEvent, ToastType, ActorEditorBackButton, ViewportSettings, ResetCameraEvent, ConfirmationRequestEvent, ActorSaveEvent, EditorAction, EditorMode, LastUsedDirectory};

#[derive(SystemParam)]
pub struct EditorEvents<'w> {
    pub reset: EventWriter<'w, ResetCameraEvent>,
    pub modal: EventWriter<'w, ConfirmationRequestEvent>,
    pub load: EventWriter<'w, ActorLoadEvent>,
    pub import: EventWriter<'w, ActorImportEvent>,
    pub save: EventWriter<'w, ActorSaveEvent>,
    pub toast: EventWriter<'w, ToastEvent>,
}

pub fn actor_editor_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ActorEditorBackButton>)>,
    mut viewport_settings: ResMut<ViewportSettings>,
    mut events: EditorEvents,
    mut slicing_settings: ResMut<SlicingSettings>,
    mut editor_mode: ResMut<EditorMode>,
    mut socket_settings: ResMut<super::super::SocketSettings>,
    current_project: Res<super::super::CurrentProject>,
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<crate::actor_editor::MainEditorCamera>>,
    mut commands: Commands,
    mut last_dir: ResMut<LastUsedDirectory>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) 
               || keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight);

    // Global Hotkeys
    if ctrl {
        if keyboard.just_pressed(KeyCode::KeyO) {
            let current_dir = std::env::current_dir().unwrap_or_default();
            let actors_dir = current_dir.join("assets").join("actors");

            let directory = last_dir.0.clone().unwrap_or(actors_dir);
            
            if let Some(path) = FileDialog::new()
                .set_title("Open Actor Project Folder")
                .set_directory(directory)
                .pick_folder() {
                
                if let Some(parent) = path.parent() {
                    last_dir.0 = Some(parent.to_path_buf());
                }

                let ron_path = path.join("actor.ron");
                if ron_path.exists() {
                    events.load.send(super::super::ActorLoadEvent(ron_path));
                } else {
                    events.toast.send(ToastEvent {
                        message: "Selected folder is not a valid project (actor.ron not found)".to_string(),
                        toast_type: ToastType::Error,
                    });
                }
            }
        }
        if keyboard.just_pressed(KeyCode::KeyI) {
            let current_dir = std::env::current_dir().unwrap_or_default();
            let assets_dir = current_dir.join("assets");

            let directory = last_dir.0.clone().unwrap_or(assets_dir);

            if let Some(path) = FileDialog::new()
                .set_title("Import Model")
                .set_directory(directory)
                .add_filter("Models", &["gltf", "glb", "obj"])
                .pick_file() {
                if let Some(parent) = path.parent() {
                    last_dir.0 = Some(parent.to_path_buf());
                }
                events.import.send(ActorImportEvent(path, true));
            }
        }
        if keyboard.just_pressed(KeyCode::KeyS) {
            if !current_project.is_saved {
                let font = asset_server.load("fonts/Roboto-Regular.ttf");
                let target_camera = camera_query.get_single().ok();
                super::super::widgets::spawn_save_modal(&mut commands, &font, &current_project.name, target_camera);
            } else {
                events.save.send(ActorSaveEvent { name: None, force: false });
            }
        }
    }

    // Mode Switching
    if keyboard.just_pressed(KeyCode::Tab) {
        *editor_mode = match *editor_mode {
            EditorMode::Slicing => EditorMode::Sockets,
            EditorMode::Sockets => EditorMode::Slicing,
        };
        events.toast.send(ToastEvent {
            message: format!("Mode: {:?}", *editor_mode),
            toast_type: ToastType::Info,
        });
    }

    if keyboard.just_pressed(KeyCode::KeyL) {
        slicing_settings.locked = !slicing_settings.locked;
        events.toast.send(ToastEvent {
            message: if slicing_settings.locked { "Slicer Locked" } else { "Slicer Unlocked" }.to_string(),
            toast_type: ToastType::Info,
        });
    }

    // Menu navigation
    let mut trigger_back = false;
    if keyboard.just_pressed(KeyCode::Escape) {
        trigger_back = true;
    }

    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            trigger_back = true;
        }
    }

    if trigger_back {
        events.modal.send(ConfirmationRequestEvent {
            title: "Discard Changes?".to_string(),
            message: "Are you sure you want to return to menu? Any unsaved changes will be lost.".to_string(),
            action: EditorAction::BackToMenu,
        });
    }

    // Viewport Hotkeys
    if keyboard.just_pressed(KeyCode::KeyG) { viewport_settings.grid = !viewport_settings.grid; }
    if keyboard.just_pressed(KeyCode::KeyS) { viewport_settings.slices = !viewport_settings.slices; }
    if keyboard.just_pressed(KeyCode::KeyK) { viewport_settings.sockets = !viewport_settings.sockets; }
    if keyboard.just_pressed(KeyCode::KeyZ) { viewport_settings.gizmos = !viewport_settings.gizmos; }
    if keyboard.just_pressed(KeyCode::KeyR) { events.reset.send(ResetCameraEvent); }

    // Fast Socket Spawn Hotkey
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        *editor_mode = EditorMode::Sockets;
        socket_settings.is_adding = true;
        events.toast.send(ToastEvent {
            message: "Socket Placement Active".to_string(),
            toast_type: ToastType::Info,
        });
    }
}
