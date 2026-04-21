use rfd::FileDialog;
use bevy::render::primitives::Aabb;
use super::ui_project::ProjectAction;
use super::{ActorImportEvent, PendingImport, OriginalMeshComponent, EditorStatus, ToastEvent, ToastType};
use bevy::prelude::*;
use crate::GameState;
use super::{ActorEditorBackButton, ViewportSettings, ResetCameraEvent, MainEditorCamera, GizmoCamera};

pub fn actor_editor_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ActorEditorBackButton>)>,
    mut viewport_settings: ResMut<ViewportSettings>,
    mut reset_events: EventWriter<ResetCameraEvent>,
    mut modal_events: EventWriter<super::ConfirmationRequestEvent>,
    mut import_events: EventWriter<ActorImportEvent>,
    mut save_events: EventWriter<super::ActorSaveEvent>,
    mut toast_events: EventWriter<ToastEvent>,
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
            save_events.send(super::ActorSaveEvent);
            toast_events.send(ToastEvent {
                message: "Save feature coming soon!".to_string(),
                toast_type: ToastType::Info,
            });
        }
        if keyboard.just_pressed(KeyCode::KeyO) {
            toast_events.send(ToastEvent {
                message: "Open project feature coming soon!".to_string(),
                toast_type: ToastType::Info,
            });
        }
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
        modal_events.send(super::ConfirmationRequestEvent {
            title: "Discard Changes?".to_string(),
            message: "Are you sure you want to return to menu? Any unsaved changes will be lost.".to_string(),
            action: super::EditorAction::BackToMenu,
        });
    }

    // Viewport Hotkeys
    if keyboard.just_pressed(KeyCode::KeyG) {
        viewport_settings.grid = !viewport_settings.grid;
    }
    if keyboard.just_pressed(KeyCode::KeyS) {
        viewport_settings.slices = !viewport_settings.slices;
    }
    if keyboard.just_pressed(KeyCode::KeyK) {
        viewport_settings.sockets = !viewport_settings.sockets;
    }
    if keyboard.just_pressed(KeyCode::KeyZ) {
        viewport_settings.gizmos = !viewport_settings.gizmos;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        reset_events.send(ResetCameraEvent);
    }
}

pub fn gizmo_sync_system(
    main_camera: Query<&Transform, (With<MainEditorCamera>, Without<GizmoCamera>)>,
    mut gizmo_camera: Query<&mut Transform, With<GizmoCamera>>,
) {
    if let Ok(main_transform) = main_camera.get_single() {
        if let Ok(mut gizmo_transform) = gizmo_camera.get_single_mut() {
            // Position gizmo camera on a sphere around origin to match main camera's relative angle
            let distance = 3.0; // Fixed distance for the gizmo UI
            let rotation = main_transform.rotation;
            
            // The gizmo camera should look at origin (0,0,0) from the same direction as main camera
            gizmo_transform.translation = rotation * (Vec3::Z * distance);
            gizmo_transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}


pub fn gizmo_viewport_system(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut gizmo_camera: Query<&mut Camera, With<GizmoCamera>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Ok(mut camera) = gizmo_camera.get_single_mut() else { return; };
    
    let size = 120;
    let padding = 20;
    
    camera.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(padding, window.physical_height().saturating_sub(size + padding)),
        physical_size: UVec2::new(size, size),
        depth: 0.0..1.0,
    });
}

pub fn status_update_system(
    status: Res<super::EditorStatus>,
    mut query: Query<&mut Text, With<super::widgets::StatusText>>,
) {
    if !status.is_changed() { return; }
    if let Ok(mut text) = query.get_single_mut() {
        let (val, color) = match *status {
            super::EditorStatus::Ready => ("READY", Color::srgb(0.8, 0.8, 0.8)),
            super::EditorStatus::Saving => ("SAVING...", Color::srgb(1.0, 0.8, 0.2)),
            super::EditorStatus::Loading => ("LOADING...", Color::srgb(0.2, 0.8, 1.0)),
            super::EditorStatus::Processing => ("PROCESSING...", Color::srgb(0.8, 0.4, 1.0)),
        };
        text.sections[0].value = val.to_string();
        text.sections[0].style.color = color;
    }
}

pub fn polycount_update_system(
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Handle<Mesh>>,
    root_query: Query<Entity, (With<super::ActorEditorEntity>, Without<super::EditorHelper>)>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut Text, With<super::widgets::PolycountText>>,
) {
    let mut total_polys = 0;
    
    for root_entity in root_query.iter() {
        let mut stack = vec![root_entity];
        while let Some(entity) = stack.pop() {
            if let Ok(handle) = mesh_query.get(entity) {
                if let Some(mesh) = meshes.get(handle) {
                    if let Some(indices) = mesh.indices() {
                        total_polys += indices.len() / 3;
                    } else if let Some(pos) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                        total_polys += pos.len() / 3;
                    }
                }
            }
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    stack.push(*child);
                }
            }
        }
    }
    
    if let Ok(mut text) = text_query.get_single_mut() {
        let new_val = format!("POLYS: {}", total_polys);
        if text.sections[0].value != new_val {
            text.sections[0].value = new_val;
        }
    }
}

pub fn toast_manager_system(
    mut commands: Commands,
    mut toast_events: EventReader<super::ToastEvent>,
    asset_server: Res<AssetServer>,
    container_query: Query<Entity, With<super::widgets::ToastContainer>>,
    mut timer_query: Query<(Entity, &mut super::widgets::ToastTimer, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");

    // Spawn new toasts
    if let Ok(container) = container_query.get_single() {
        for event in toast_events.read() {
            commands.entity(container).with_children(|p| {
                super::widgets::spawn_toast_item(p, &font, &icon_font, &event.message, event.toast_type);
            });
        }
    }

    // Update timers and despawn
    for (entity, mut timer, mut bg) in timer_query.iter_mut() {
        timer.0.tick(time.delta());
        
        // Simple fade out in last 0.5s
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
    mut modal_events: EventReader<super::ConfirmationRequestEvent>,
    asset_server: Res<AssetServer>,
    cancel_query: Query<&Interaction, (Changed<Interaction>, With<super::widgets::CancelModalButton>)>,
    confirm_query: Query<(&Interaction, &super::widgets::ConfirmModalButton), (Changed<Interaction>, With<super::widgets::ConfirmModalButton>)>,
    overlay_query: Query<Entity, With<super::widgets::ModalOverlay>>,
    camera_query: Query<Entity, With<super::MainEditorCamera>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let icon_font = asset_server.load("fonts/forkawesome.ttf");
    let target_camera = camera_query.get_single().ok();

    // Spawn modal
    for event in modal_events.read() {
        super::widgets::spawn_confirmation_modal(&mut commands, &font, &icon_font, &event.title, &event.message, event.action, target_camera);
    }

    // Handle Cancel
    for interaction in cancel_query.iter() {
        if *interaction == Interaction::Pressed {
            for entity in overlay_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    // Handle Confirm
    for (interaction, confirm) in confirm_query.iter() {
        if *interaction == Interaction::Pressed {
            match confirm.0 {
                super::EditorAction::BackToMenu => {
                    next_state.set(GameState::Menu);
                }
            }
            // Close modal
            for entity in overlay_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn color_picker_system(
    mut color_res: ResMut<super::EditorMaterialColor>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<super::widgets::ColorPickerButton>)>,
    hue_query: Query<(&Interaction, &Node, &GlobalTransform), With<super::widgets::ColorHueSlider>>,
    preset_query: Query<(&Interaction, &super::widgets::ColorPreset)>,
    mut container_query: Query<&mut Style, With<super::widgets::ColorPickerContainer>>,
    mut preview_query: Query<&mut BackgroundColor, (With<super::widgets::ColorPickerButton>, Without<super::widgets::ColorPreset>)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    // Toggle
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            color_res.is_open = !color_res.is_open;
            if let Ok(mut style) = container_query.get_single_mut() {
                style.display = if color_res.is_open { Display::Flex } else { Display::None };
            }
        }
    }

    // Hue Slider
    let Ok(window) = window_query.get_single() else { return; };
    if let Some(cursor) = window.cursor_position() {
        for (interaction, node, transform) in hue_query.iter() {
            if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
                if *interaction == Interaction::Pressed {
                    let rect = node.size();
                    let pos = transform.translation().truncate();
                    let local_x = cursor.x - (pos.x - rect.x / 2.0);
                    let hue = (local_x / rect.x).clamp(0.0, 1.0) * 360.0;
                    color_res.hue = hue;
                    color_res.color = Color::hsla(hue, 0.8, 0.5, 1.0);
                }
            }
        }
    }

    // Presets
    for (interaction, preset) in preset_query.iter() {
        if *interaction == Interaction::Pressed {
            color_res.color = preset.0;
        }
    }

    // Update Preview
    if color_res.is_changed() {
        if let Ok(mut bg) = preview_query.get_single_mut() {
            bg.0 = color_res.color;
        }
    }
}

pub fn material_sync_system(
    color_res: Res<super::EditorMaterialColor>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_query: Query<&Handle<StandardMaterial>, With<super::ActorEditorEntity>>,
) {
    if !color_res.is_changed() { return; }
    for handle in mesh_query.iter() {
        if let Some(mat) = materials.get_mut(handle) {
            mat.base_color = color_res.color;
        }
    }
}

pub fn actor_import_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ProjectAction>)>,
    mut import_events: EventWriter<ActorImportEvent>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(path) = FileDialog::new()
                .add_filter("Models", &["gltf", "glb", "obj"])
                .pick_file() {
                import_events.send(ActorImportEvent(path));
            }
        }
    }
}

pub fn actor_import_event_system(
    mut events: EventReader<ActorImportEvent>,
    asset_server: Res<AssetServer>,
    mut status: ResMut<EditorStatus>,
    mut pending: ResMut<PendingImport>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in events.read() {
        let path = &event.0;
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

        *status = EditorStatus::Loading;
        
        if relative_path.ends_with(".obj") {
            pending.mesh_handle = Some(asset_server.load(relative_path));
            pending.handle = None;
        } else {
            // For GLTF/GLB we need to specify a label to load it as a Scene
            let scene_path = format!("{}#Scene0", relative_path);
            pending.handle = Some(asset_server.load(scene_path));
            pending.mesh_handle = None;
        }
    }
}

pub fn import_loading_overlay_system(
    status: Res<EditorStatus>,
    mut query: Query<&mut Style, With<super::widgets::LoadingOverlay>>,
) {
    if !status.is_changed() { return; }
    if let Ok(mut style) = query.get_single_mut() {
        style.display = if *status == EditorStatus::Loading {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn actor_import_processing_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingImport>,
    mut status: ResMut<EditorStatus>,
    _meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut progress: ResMut<super::ImportProgress>,
    time: Res<Time>,
    mut toast_events: EventWriter<ToastEvent>,
    actor_entities: Query<Entity, (With<super::ActorEditorEntity>, Without<Camera>, Without<Node>, Without<super::EditorHelper>)>,
) {
    if *status != EditorStatus::Loading { return; }

    let mut target_progress = progress.0;
    let mut finished = false;
    let mut loaded_mesh: Option<Handle<Mesh>> = None;

    if let Some(ref handle) = pending.mesh_handle {
        match asset_server.get_load_state(handle) {
            Some(bevy::asset::LoadState::Loaded) => {
                target_progress = 0.7; // Transition to processing
                finished = true;
                loaded_mesh = Some(handle.clone());
            }
            Some(bevy::asset::LoadState::Loading) => {
                // Smoothly approach 0.7
                target_progress = (progress.0 + time.delta_seconds() * 0.1).min(0.65);
            }
            Some(bevy::asset::LoadState::Failed(_)) => {
                *status = EditorStatus::Ready;
                progress.0 = 0.0;
                pending.mesh_handle = None;
                toast_events.send(ToastEvent {
                    message: "Failed to load OBJ model".to_string(),
                    toast_type: ToastType::Error,
                });
                return;
            }
            _ => {}
        }
    } else if let Some(ref handle) = pending.handle {
        match asset_server.get_load_state(handle) {
            Some(bevy::asset::LoadState::Loaded) => {
                target_progress = 0.7;
                finished = true;
            }
            Some(bevy::asset::LoadState::Loading) => {
                // For GLTF we use a more detailed smooth progress
                target_progress = (progress.0 + time.delta_seconds() * 0.05).min(0.68);
            }
            Some(bevy::asset::LoadState::Failed(_)) => {
                *status = EditorStatus::Ready;
                progress.0 = 0.0;
                pending.handle = None;
                toast_events.send(ToastEvent {
                    message: "Failed to load GLTF model".to_string(),
                    toast_type: ToastType::Error,
                });
                return;
            }
            _ => {}
        }
    }

    progress.0 = target_progress;

    if finished {
        for entity in actor_entities.iter() {
            commands.entity(entity).despawn_recursive();
        }

        if let Some(mesh_handle) = loaded_mesh {
            commands.spawn((
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        ..default()
                    }),
                    ..default()
                },
                super::ActorEditorEntity,
                super::AwaitingNormalization,
                OriginalMeshComponent(mesh_handle),
            ));

            toast_events.send(ToastEvent {
                message: "Model file loaded".to_string(),
                toast_type: ToastType::Success,
            });
        } else if pending.handle.is_some() {
             commands.spawn((
                SceneBundle {
                    scene: pending.handle.clone().unwrap(),
                    ..default()
                },
                super::ActorEditorEntity,
                super::AwaitingNormalization,
            ));
            
            toast_events.send(ToastEvent {
                message: "Scene file loaded".to_string(),
                toast_type: ToastType::Success,
            });
        }

        *status = EditorStatus::Processing;
        pending.handle = None;
        pending.mesh_handle = None;
    }
}

pub fn progress_bar_update_system(
    progress: Res<super::ImportProgress>,
    mut progress_fill: Query<&mut Style, With<super::widgets::ProgressBarFill>>,
    mut progress_text: Query<&mut Text, With<super::widgets::ProgressBarText>>,
) {
    if !progress.is_changed() { return; }
    
    if let Ok(mut style) = progress_fill.get_single_mut() {
        style.width = Val::Percent(progress.0 * 100.0);
    }
    if let Ok(mut text) = progress_text.get_single_mut() {
        text.sections[0].value = format!("{:.0}%", progress.0 * 100.0);
    }
}

pub fn normalization_system(
    mut commands: Commands,
    query: Query<Entity, With<super::AwaitingNormalization>>,
    mut state_query: Query<(Entity, &mut super::NormalizationState)>,
    children_query: Query<&Children>,
    mesh_query: Query<(&Aabb, &GlobalTransform, &Handle<Mesh>)>,
    mut progress: ResMut<super::ImportProgress>,
    mut status: ResMut<super::EditorStatus>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    // 1. Initial Discovery
    for root_entity in query.iter() {
        let mut stack = vec![root_entity];
        let mut entities_to_process = Vec::new();
        
        while let Some(entity) = stack.pop() {
            entities_to_process.push(entity);
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    stack.push(*child);
                }
            }
        }
        
        commands.entity(root_entity).remove::<super::AwaitingNormalization>();
        commands.entity(root_entity).insert(super::NormalizationState {
            entities_to_process,
            processed_count: 0,
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
            found_meshes: Vec::new(),
        });
        
        progress.0 = 0.7;
        *status = super::EditorStatus::Processing;
    }

    // 2. Incremental Processing
    for (root_entity, mut state) in state_query.iter_mut() {
        // Process a chunk of entities per frame (e.g., 50)
        let chunk_size = 50;
        let mut processed_this_frame = 0;
        
        while processed_this_frame < chunk_size && state.processed_count < state.entities_to_process.len() {
            let entity = state.entities_to_process[state.processed_count];
            
            if let Ok((aabb, transform, mesh_handle)) = mesh_query.get(entity) {
                let matrix = transform.compute_matrix();
                let world_aabb = Aabb {
                    center: matrix.transform_point3a(aabb.center),
                    half_extents: matrix.transform_vector3a(aabb.half_extents).abs(),
                };
                
                let aabb_min = Vec3::from(world_aabb.center - world_aabb.half_extents);
                let aabb_max = Vec3::from(world_aabb.center + world_aabb.half_extents);
                
                state.min = state.min.min(aabb_min);
                state.max = state.max.max(aabb_max);
                state.found_meshes.push((entity, mesh_handle.clone()));
            }
            
            state.processed_count += 1;
            processed_this_frame += 1;
        }

        // Update progress (0.7 -> 0.95)
        let ratio = state.processed_count as f32 / state.entities_to_process.len() as f32;
        progress.0 = 0.7 + ratio * 0.25;

        // 3. Finalization
        if state.processed_count >= state.entities_to_process.len() {
            let found = !state.found_meshes.is_empty();
            if found {
                let center = (state.min + state.max) / 2.0;
                let size = state.max - state.min;
                let max_dim = size.x.max(size.y).max(size.z);
                
                if max_dim > 0.0 {
                    let scale = 2.0 / max_dim;
                    let offset = -center;
                    let y_offset = size.y * 0.5;
                    
                    commands.entity(root_entity).insert(Transform {
                        translation: (offset + Vec3::Y * y_offset) * scale,
                        scale: Vec3::splat(scale),
                        rotation: Quat::IDENTITY,
                    });
                    
                    for (entity, handle) in &state.found_meshes {
                        commands.entity(*entity).insert(super::OriginalMeshComponent(handle.clone()));
                    }
                    
                    toast_events.send(ToastEvent {
                        message: "Actor ready for editing".to_string(),
                        toast_type: ToastType::Success,
                    });
                }
            }
            
            commands.entity(root_entity).remove::<super::NormalizationState>();
            progress.0 = 1.0;
            *status = super::EditorStatus::Ready;
        }
    }
}

pub fn gizmo_label_billboard_system(
    gizmo_camera: Query<&Transform, (With<GizmoCamera>, Without<super::GizmoLabel>)>,
    mut labels: Query<&mut Transform, With<super::GizmoLabel>>,
) {
    if let Ok(camera_transform) = gizmo_camera.get_single() {
        for mut label_transform in labels.iter_mut() {
            label_transform.rotation = camera_transform.rotation;
        }
    }
}
