use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use rfd::FileDialog;
use super::super::{EditorStatus, ActorBounds, ActorEditorEntity, Actor3DRoot, EditorHelper, ToastEvent, ToastType, ActorImportEvent, ActorSaveEvent, ActorLoadEvent, PendingImport, ImportProgress, GameState, SlicingSettings, ConfirmationRequestEvent, EditorAction, EditorMaterialColor, EditorMode, ViewportSettings, CurrentProject, LastUsedDirectory};
use super::super::ui_project::{ModeTab, ProjectModeContent};
use super::super::ui::inspector::types::{SocketsSectionMarker, PartsSectionMarker, SelectedSocket};
use super::super::widgets::CollapsibleSection;

pub fn status_update_system(
    status: Res<EditorStatus>,
    mut query: Query<&mut Text, With<super::super::widgets::StatusText>>,
) {
    if !status.is_changed() { return; }
    if let Ok(mut text) = query.get_single_mut() {
        let (val, color) = match *status {
            EditorStatus::Ready => ("READY", Color::srgb(0.8, 0.8, 0.8)),
            EditorStatus::Saving => ("SAVING...", Color::srgb(1.0, 0.8, 0.2)),
            EditorStatus::Loading => ("LOADING...", Color::srgb(0.2, 0.8, 1.0)),
            EditorStatus::Processing => ("PROCESSING...", Color::srgb(0.8, 0.4, 1.0)),
        };
        text.sections[0].value = val.to_string();
        text.sections[0].style.color = color;
    }
}

pub fn polycount_update_system(
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Handle<Mesh>>,
    root_query: Query<(Entity, Option<&ActorBounds>), (With<Actor3DRoot>, Without<EditorHelper>)>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut Text, With<super::super::widgets::PolycountText>>,
) {
    let mut total_polys = 0;
    let mut original_polys = 0;
    
    for (root_entity, bounds_opt) in root_query.iter() {
        if let Some(bounds) = bounds_opt { original_polys = bounds.original_polys; }

        let mut stack = vec![root_entity];
        while let Some(entity) = stack.pop() {
            if let Ok(handle) = mesh_query.get(entity) {
                if let Some(mesh) = meshes.get(handle) {
                    if let Some(indices) = mesh.indices() {
                        total_polys += indices.len() / 3;
                    } else if let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                        total_polys += pos.len() / 3;
                    }
                }
            }
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() { stack.push(*child); }
            }
        }
    }
    
    if let Ok(mut text) = text_query.get_single_mut() {
        if original_polys > 0 {
            text.sections[0].value = format!("POLYS: {} / ORIG: {}", total_polys, original_polys);
        } else {
            text.sections[0].value = format!("POLYS: {}", total_polys);
        }
    }
}

pub fn toast_manager_system(
    mut commands: Commands,
    mut toast_events: EventReader<ToastEvent>,
    asset_server: Res<AssetServer>,
    container_query: Query<Entity, With<super::super::widgets::ToastContainer>>,
    mut timer_query: Query<(Entity, &mut super::super::widgets::ToastTimer, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");

    if let Ok(container) = container_query.get_single() {
        for event in toast_events.read() {
            commands.entity(container).with_children(|p| {
                super::super::widgets::spawn_toast_item(p, &font, &icon_font, &event.message, event.toast_type);
            });
        }
    }

    for (entity, mut timer, mut bg) in timer_query.iter_mut() {
        timer.0.tick(time.delta());
        let rem = timer.0.remaining_secs();
        if rem < 0.5 {
            let alpha = (rem / 0.5).clamp(0.0, 1.0);
            bg.0.set_alpha(alpha * 0.95);
        }
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn modal_manager_system(
    mut commands: Commands,
    mut modal_events: EventReader<ConfirmationRequestEvent>,
    asset_server: Res<AssetServer>,
    cancel_query: Query<&Interaction, (Changed<Interaction>, With<super::super::widgets::CancelModalButton>)>,
    confirm_query: Query<(&Interaction, &super::super::widgets::ConfirmModalButton), (Changed<Interaction>, With<super::super::widgets::ConfirmModalButton>)>,
    overlay_query: Query<Entity, With<super::super::widgets::ModalOverlay>>,
    camera_query: Query<Entity, With<crate::actor_editor::MainEditorCamera>>,
    mut next_state: ResMut<NextState<GameState>>,
    input_query: Query<&super::super::widgets::text_input::TextInput, With<super::super::SaveModalInput>>,
    mut save_events: EventWriter<ActorSaveEvent>,
) {
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");
    let target_camera = camera_query.get_single().ok();

    for event in modal_events.read() {
        super::super::widgets::spawn_confirmation_modal(&mut commands, &font, &icon_font, &event.title, &event.message, event.action.clone(), target_camera);
    }

    for interaction in cancel_query.iter() {
        if *interaction == Interaction::Pressed {
            for entity in overlay_query.iter() { commands.entity(entity).despawn_recursive(); }
        }
    }

    for (interaction, confirm) in confirm_query.iter() {
        if *interaction == Interaction::Pressed {
            match &confirm.0 { 
                EditorAction::BackToMenu => { next_state.set(GameState::Menu); } 
                EditorAction::SaveProject(_) => {
                    if let Ok(input) = input_query.get_single() {
                        let name = input.value.trim();
                        if !name.is_empty() {
                            save_events.send(ActorSaveEvent { name: Some(name.to_string()), force: false });
                        }
                    }
                }
                EditorAction::OverwriteProject(name) => {
                    save_events.send(ActorSaveEvent { name: Some(name.clone()), force: true });
                }
            }
            for entity in overlay_query.iter() { commands.entity(entity).despawn_recursive(); }
        }
    }
}

pub fn color_picker_system(
    mut color_res: ResMut<EditorMaterialColor>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<super::super::widgets::ColorPickerButton>)>,
    hue_query: Query<(&Interaction, &Node, &GlobalTransform), With<super::super::widgets::ColorHueSlider>>,
    preset_query: Query<(&Interaction, &super::super::widgets::ColorPreset)>,
    mut container_query: Query<&mut Style, With<super::super::widgets::ColorPickerContainer>>,
    mut preview_query: Query<&mut BackgroundColor, (With<super::super::widgets::ColorPickerButton>, Without<super::super::widgets::ColorPreset>)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut initial_color: Local<Option<Color>>,
    mut action_stack: ResMut<super::super::systems::undo_redo::ActionStack>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            color_res.is_open = !color_res.is_open;
            if let Ok(mut style) = container_query.get_single_mut() {
                style.display = if color_res.is_open { Display::Flex } else { Display::None };
            }
        }
    }

    let Ok(window) = window_query.get_single() else { return; };
    if let Some(cursor) = window.cursor_position() {
        for (interaction, node, transform) in hue_query.iter() {
            if *interaction == Interaction::Pressed {
                if initial_color.is_none() {
                    *initial_color = Some(color_res.color);
                }
                let rect = node.size();
                let pos = transform.translation().truncate();
                let local_x = cursor.x - (pos.x - rect.x / 2.0);
                let hue = (local_x / rect.x).clamp(0.0, 1.0) * 360.0;
                color_res.hue = hue;
                color_res.color = Color::hsla(hue, 0.8, 0.5, 1.0);
            } else if initial_color.is_some() {
                // Drag finished
                let old = initial_color.unwrap();
                let new = color_res.color;
                if old != new {
                    action_stack.push(Box::new(super::super::systems::undo_redo::ChangeMaterialColorCommand {
                        old_color: old,
                        new_color: new,
                    }));
                }
                *initial_color = None;
            }
        }
    }

    for (interaction, preset) in preset_query.iter() {
        if *interaction == Interaction::Pressed { 
            let old = color_res.color;
            let new = preset.0;
            if old != new {
                color_res.color = new; 
                action_stack.push(Box::new(super::super::systems::undo_redo::ChangeMaterialColorCommand {
                    old_color: old,
                    new_color: new,
                }));
            }
        }
    }

    if color_res.is_changed() {
        if let Ok(mut bg) = preview_query.get_single_mut() { bg.0 = color_res.color; }
    }
}

pub fn material_sync_system(
    color_res: Res<EditorMaterialColor>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_query: Query<&Handle<StandardMaterial>, (With<ActorEditorEntity>, Without<EditorHelper>)>,
) {
    if !color_res.is_changed() { return; }
    for handle in mesh_query.iter() {
        if let Some(mat) = materials.get_mut(handle) { mat.base_color = color_res.color; }
    }
}

pub fn project_action_system(
    interaction_query: Query<(&Interaction, &super::super::ui_project::ProjectAction), Changed<Interaction>>,
    mut import_events: EventWriter<ActorImportEvent>,
    mut load_events: EventWriter<ActorLoadEvent>,
    mut save_events: EventWriter<ActorSaveEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    current_project: Res<CurrentProject>,
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<crate::actor_editor::MainEditorCamera>>,
    mut commands: Commands,
    mut last_dir: ResMut<LastUsedDirectory>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                super::super::ui_project::ProjectAction::Import => {
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
                        import_events.send(ActorImportEvent(path, true));
                    }
                }
                super::super::ui_project::ProjectAction::Save => {
                    if !current_project.is_saved {
                        let font = asset_server.load("fonts/Roboto-Regular.ttf");
                        let target_camera = camera_query.get_single().ok();
                        super::super::widgets::spawn_save_modal(&mut commands, &font, &current_project.name, target_camera);
                    } else {
                        save_events.send(ActorSaveEvent { name: None, force: false });
                    }
                }
                super::super::ui_project::ProjectAction::Open => {
                    let current_dir = std::env::current_dir().unwrap_or_default();
                    let actors_dir = current_dir.join("assets").join("actors");
                    
                    let directory = last_dir.0.clone().unwrap_or(actors_dir);
                    
                    if let Some(path) = FileDialog::new()
                        .set_title("Open Actor Project Folder")
                        .set_directory(directory)
                        .pick_folder() {
                        
                        // Remember this directory (the parent of the selected project folder)
                        if let Some(parent) = path.parent() {
                            last_dir.0 = Some(parent.to_path_buf());
                        }
                        
                        let ron_path = path.join("actor.ron");
                        if ron_path.exists() {
                            load_events.send(super::super::ActorLoadEvent(ron_path));
                        } else {
                            toast_events.send(ToastEvent {
                                message: "Selected folder is not a valid project (actor.ron not found)".to_string(),
                                toast_type: ToastType::Error,
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn actor_import_event_system(
    mut events: EventReader<ActorImportEvent>,
    asset_server: Res<AssetServer>,
    mut status: ResMut<EditorStatus>,
    mut pending: ResMut<PendingImport>,
    mut current_project: ResMut<CurrentProject>,
    mut slicing_settings: ResMut<SlicingSettings>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in events.read() {
        let path = &event.0;
        let reset_slicing = event.1;

        if reset_slicing {
            *slicing_settings = SlicingSettings::default();
            // Explicitly set the initial slice values for new models
            slicing_settings.top_cut = 0.75;
            slicing_settings.bottom_cut = 0.25;
            slicing_settings.last_top = -1.0; // Keep this to trigger the initial slice logic if needed, but the values are now correct
        }

        let current_dir = std::env::current_dir().unwrap_or_default();
        let assets_dir = current_dir.join("assets");
        
        let relative_path = if let Ok(rel) = path.strip_prefix(&assets_dir) {
            rel.to_string_lossy().to_string()
        } else {
            toast_events.send(ToastEvent {
                message: "Please select a file inside the project assets folder".to_string(),
                toast_type: ToastType::Error,
            });
            continue;
        };

        // Update project info
        let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unnamed");
        current_project.name = file_name.to_string();
        current_project.source_path = relative_path.clone();
        current_project.is_saved = false;

        *status = EditorStatus::Loading;
        
        if relative_path.ends_with(".obj") {
            pending.mesh_handle = Some(asset_server.load(relative_path));
            pending.handle = None;
        } else {
            let scene_path = format!("{}#Scene0", relative_path);
            pending.handle = Some(asset_server.load(scene_path));
            pending.mesh_handle = None;
        }
    }
}

pub fn import_loading_overlay_system(
    status: Res<EditorStatus>,
    mut query: Query<&mut Style, With<super::super::widgets::LoadingOverlay>>,
) {
    if !status.is_changed() { return; }
    if let Ok(mut style) = query.get_single_mut() {
        style.display = if *status == EditorStatus::Loading { Display::Flex } else { Display::None };
    }
}

pub fn actor_import_processing_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingImport>,
    mut status: ResMut<EditorStatus>,
    _meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut progress: ResMut<ImportProgress>,
    time: Res<Time>,
    mut toast_events: EventWriter<ToastEvent>,
    actor_entities: Query<Entity, (With<ActorEditorEntity>, Without<Camera>, Without<Node>, Without<EditorHelper>)>,
) {
    if *status != EditorStatus::Loading { return; }

    let mut target_progress = progress.0;
    let mut finished = false;
    let mut loaded_mesh: Option<Handle<Mesh>> = None;

    if let Some(ref handle) = pending.mesh_handle {
        match asset_server.get_load_state(handle) {
            Some(bevy::asset::LoadState::Loaded) => {
                target_progress = 0.7;
                finished = true;
                loaded_mesh = Some(handle.clone());
            }
            Some(bevy::asset::LoadState::Loading) => { target_progress = (progress.0 + time.delta_seconds() * 0.1).min(0.65); }
            Some(bevy::asset::LoadState::Failed(_)) => {
                *status = EditorStatus::Ready; progress.0 = 0.0; pending.mesh_handle = None;
                toast_events.send(ToastEvent { message: "Failed to load OBJ model".to_string(), toast_type: ToastType::Error });
                return;
            }
            _ => {}
        }
    } else if let Some(ref handle) = pending.handle {
        match asset_server.get_load_state(handle) {
            Some(bevy::asset::LoadState::Loaded) => { target_progress = 0.7; finished = true; }
            Some(bevy::asset::LoadState::Loading) => { target_progress = (progress.0 + time.delta_seconds() * 0.05).min(0.68); }
            Some(bevy::asset::LoadState::Failed(_)) => {
                *status = EditorStatus::Ready; progress.0 = 0.0; pending.handle = None;
                toast_events.send(ToastEvent { message: "Failed to load GLTF model".to_string(), toast_type: ToastType::Error });
                return;
            }
            _ => {}
        }
    }

    progress.0 = target_progress;

    if finished {
        for entity in actor_entities.iter() { commands.entity(entity).despawn_recursive(); }

        if let Some(mesh_handle) = loaded_mesh {
            commands.spawn((
                SpatialBundle::default(),
                ActorEditorEntity, 
                Actor3DRoot,
                crate::actor_editor::AwaitingNormalization,
            )).with_children(|p| {
                p.spawn(PbrBundle { 
                    mesh: mesh_handle.clone(), 
                    material: materials.add(StandardMaterial { base_color: Color::WHITE, ..default() }), 
                    ..default() 
                });
            });
        } else if pending.handle.is_some() {
             commands.spawn(( SceneBundle { scene: pending.handle.clone().unwrap(), ..default() }, ActorEditorEntity, Actor3DRoot, crate::actor_editor::AwaitingNormalization, ));
        }
        *status = EditorStatus::Processing;
        pending.handle = None;
        pending.mesh_handle = None;
    }
}

pub fn progress_bar_update_system(
    progress: Res<ImportProgress>,
    mut progress_fill: Query<&mut Style, With<super::super::widgets::ProgressBarFill>>,
    mut progress_text: Query<&mut Text, With<super::super::widgets::ProgressBarText>>,
) {
    if !progress.is_changed() { return; }
    if let Ok(mut style) = progress_fill.get_single_mut() { style.width = Val::Percent(progress.0 * 100.0); }
    if let Ok(mut text) = progress_text.get_single_mut() { text.sections[0].value = format!("{:.0}%", progress.0 * 100.0); }
}

pub fn slicer_lock_system(
    mut slicing_settings: ResMut<SlicingSettings>,
    mut button_query: Query<(Ref<Interaction>, &mut BackgroundColor, &Children), With<super::super::widgets::SlicerLockButton>>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut bg, children) in button_query.iter_mut() {
        if interaction.is_changed() && *interaction == Interaction::Pressed {
            slicing_settings.locked = !slicing_settings.locked;
        }
        
        let (color, icon) = if slicing_settings.locked {
            (Color::srgb(0.8, 0.2, 0.2), "\u{f023}") // Red / Locked
        } else {
            (Color::srgb(0.2, 0.8, 0.2), "\u{f09c}") // Green / Unlocked
        };

        *bg = color.with_alpha(if *interaction == Interaction::Hovered { 0.9 } else { 0.7 }).into();

        if let Ok(mut text) = text_query.get_mut(children[0]) {
            text.sections[0].value = icon.to_string();
        }
    }
}

pub fn mode_tab_interaction_system(
    mut editor_mode: ResMut<EditorMode>,
    interaction_query: Query<(&Interaction, &ModeTab), (Changed<Interaction>, With<ModeTab>)>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for (interaction, tab) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if *editor_mode != tab.0 {
                *editor_mode = tab.0;
                toast_events.send(ToastEvent {
                    message: format!("Mode: {:?}", *editor_mode),
                    toast_type: ToastType::Info,
                });
            }
        }
    }
}

pub fn mode_visual_sync_system(
    editor_mode: Res<EditorMode>,
    mut tab_query: Query<(&mut BackgroundColor, &ModeTab)>,
) {
    if !editor_mode.is_changed() { return; }
    for (mut bg, tab) in tab_query.iter_mut() {
        let is_active = *editor_mode == tab.0;
        bg.0 = if is_active { Color::srgba(0.3, 0.5, 1.0, 0.3) } else { Color::srgba(1.0, 1.0, 1.0, 0.05) };
    }
}

pub fn mode_content_visibility_system(
    editor_mode: Res<EditorMode>,
    mut content_query: Query<(&mut Style, &ProjectModeContent)>,
) {
    if !editor_mode.is_changed() { return; }
    for (mut style, content) in content_query.iter_mut() {
        style.display = if *editor_mode == content.0 { Display::Flex } else { Display::None };
    }
}

pub fn inspector_section_sync_system(
    mut editor_mode: ResMut<EditorMode>,
    mut viewport_settings: ResMut<ViewportSettings>,
    mut selected_socket: ResMut<SelectedSocket>,
    sockets_marker: Query<&Parent, With<SocketsSectionMarker>>,
    parts_marker: Query<&Parent, With<PartsSectionMarker>>,
    mut section_set: ParamSet<(
        Query<Ref<CollapsibleSection>>,
        Query<&mut CollapsibleSection>,
    )>,
) {
    let Ok(sockets_p) = sockets_marker.get_single() else { return; };
    let Ok(parts_p) = parts_marker.get_single() else { return; };

    let sockets_entity = sockets_p.get();
    let parts_entity = parts_p.get();

    // 1. Sync from UI to Resource: Check if user manually opened a section
    let mut target_mode = *editor_mode;

    {
        let section_ref_query = section_set.p0();
        if let Ok(sockets) = section_ref_query.get(sockets_entity) {
            if sockets.is_changed() && sockets.is_open && *editor_mode != EditorMode::Sockets {
                target_mode = EditorMode::Sockets;
            }
        }
        if let Ok(parts) = section_ref_query.get(parts_entity) {
            if parts.is_changed() && parts.is_open && *editor_mode != EditorMode::Slicing {
                target_mode = EditorMode::Slicing;
            }
        }
    }

    if *editor_mode != target_mode {
        *editor_mode = target_mode;
        // Clear selection when leaving Sockets mode
        if target_mode == EditorMode::Slicing {
            selected_socket.0 = vec![];
        }
    }

    // 2. Sync from Resource to UI: Ensure only the active mode's section is open
    if editor_mode.is_changed() {
        let current_mode = *editor_mode;

        // Auto-enable socket visibility in viewport when entering Sockets mode
        if current_mode == EditorMode::Sockets {
            viewport_settings.sockets = true;
        }

        // Sync Sockets section
        let mut needs_sockets_update = false;
        {
            let section_ref_query = section_set.p0();
            if let Ok(sockets) = section_ref_query.get(sockets_entity) {
                if sockets.is_open != (current_mode == EditorMode::Sockets) {
                    needs_sockets_update = true;
                }
            }
        }
        if needs_sockets_update {
            if let Ok(mut s_mut) = section_set.p1().get_mut(sockets_entity) {
                s_mut.is_open = current_mode == EditorMode::Sockets;
            }
        }

        // Sync Parts section
        let mut needs_parts_update = false;
        {
            let section_ref_query = section_set.p0();
            if let Ok(parts) = section_ref_query.get(parts_entity) {
                if parts.is_open != (current_mode == EditorMode::Slicing) {
                    needs_parts_update = true;
                }
            }
        }
        if needs_parts_update {
            if let Ok(mut p_mut) = section_set.p1().get_mut(parts_entity) {
                p_mut.is_open = current_mode == EditorMode::Slicing;
            }
        }
    }
}
