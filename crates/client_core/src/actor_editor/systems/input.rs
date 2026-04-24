use bevy::prelude::*;
use rfd::FileDialog;
use super::super::{ActorImportEvent, SlicingSettings, ToastEvent, ToastType, ActorEditorBackButton, ViewportSettings, ResetCameraEvent, ConfirmationRequestEvent, ActorSaveEvent, EditorAction, EditorMode};

pub fn actor_editor_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ActorEditorBackButton>)>,
    mut viewport_settings: ResMut<ViewportSettings>,
    mut reset_events: EventWriter<ResetCameraEvent>,
    mut modal_events: EventWriter<ConfirmationRequestEvent>,
    mut import_events: EventWriter<ActorImportEvent>,
    mut save_events: EventWriter<ActorSaveEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    mut slicing_settings: ResMut<SlicingSettings>,
    mut editor_mode: ResMut<EditorMode>,
    mut socket_settings: ResMut<super::super::SocketSettings>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) 
               || keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight);

    // Global Hotkeys
    if ctrl {
        if keyboard.just_pressed(KeyCode::KeyI) {
            if let Some(path) = FileDialog::new()
                .add_filter("Models", &["gltf", "glb", "obj"])
                .pick_file() {
                import_events.send(ActorImportEvent(path));
            }
        }
        if keyboard.just_pressed(KeyCode::KeyS) {
            save_events.send(ActorSaveEvent);
        }
    }

    // Mode Switching
    if keyboard.just_pressed(KeyCode::Tab) {
        *editor_mode = match *editor_mode {
            EditorMode::Slicing => EditorMode::Sockets,
            EditorMode::Sockets => EditorMode::Slicing,
        };
        toast_events.send(ToastEvent {
            message: format!("Mode: {:?}", *editor_mode),
            toast_type: ToastType::Info,
        });
    }

    if keyboard.just_pressed(KeyCode::KeyL) {
        slicing_settings.locked = !slicing_settings.locked;
        toast_events.send(ToastEvent {
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
        modal_events.send(ConfirmationRequestEvent {
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
    if keyboard.just_pressed(KeyCode::KeyR) { reset_events.send(ResetCameraEvent); }

    // Fast Socket Spawn Hotkey
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        *editor_mode = EditorMode::Sockets;
        socket_settings.is_adding = true;
        toast_events.send(ToastEvent {
            message: "Socket Placement Active".to_string(),
            toast_type: ToastType::Info,
        });
    }
}
