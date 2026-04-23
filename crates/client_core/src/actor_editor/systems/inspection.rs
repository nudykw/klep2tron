use bevy::prelude::*;
use super::super::{InspectionSettings, ActorPart, InspectionFocusEvent};
use super::super::ui_inspector::{PartFocusButton, PartSoloButton, InspectionToggle, InspectionToggleType, InspectionMasterToggle, PartsSectionMarker};
use super::super::widgets::CollapsibleSection;
use bevy_panorbit_camera::PanOrbitCamera;

pub fn inspection_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<InspectionSettings>,
    mut focus_events: EventWriter<InspectionFocusEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) && !keyboard.pressed(KeyCode::ControlLeft) && !keyboard.pressed(KeyCode::ControlRight) {
        settings.is_active = true;
        
        // Cycle through parts: None -> Head -> Body -> Engine -> None
        let next = match settings.isolated_part {
            None => Some(ActorPart::Head),
            Some(ActorPart::Head) => Some(ActorPart::Body),
            Some(ActorPart::Body) => Some(ActorPart::Engine),
            Some(ActorPart::Engine) => None,
        };
        settings.isolated_part = next;
        
        if let Some(part) = next {
            focus_events.send(InspectionFocusEvent(part));
        }
    }
}

pub fn inspection_visibility_system(
    settings: Res<InspectionSettings>,
    mut part_query: Query<(&ActorPart, &mut Visibility, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !settings.is_changed() { return; }

    for (part, mut visibility, mat_handle) in part_query.iter_mut() {
        if !settings.is_active {
            // Restore default visibility
            *visibility = Visibility::Visible;
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.base_color = mat.base_color.with_alpha(1.0);
                mat.alpha_mode = AlphaMode::Opaque;
            }
            continue;
        }

        if let Some(isolated) = settings.isolated_part {
            if *part == isolated {
                *visibility = Visibility::Visible;
                if let Some(mat) = materials.get_mut(mat_handle) {
                    mat.base_color = mat.base_color.with_alpha(1.0);
                    mat.alpha_mode = AlphaMode::Opaque;
                }
            } else {
                if settings.ghost_mode {
                    *visibility = Visibility::Visible;
                    if let Some(mat) = materials.get_mut(mat_handle) {
                        mat.base_color = mat.base_color.with_alpha(0.1);
                        mat.alpha_mode = AlphaMode::Blend;
                    }
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        } else {
            *visibility = Visibility::Visible;
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.base_color = mat.base_color.with_alpha(1.0);
                mat.alpha_mode = AlphaMode::Opaque;
            }
        }
    }
}

pub fn inspection_camera_focus_system(
    mut focus_events: EventReader<InspectionFocusEvent>,
    mut camera_query: Query<&mut PanOrbitCamera>,
    actor_query: Query<(&ActorPart, &GlobalTransform, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
) {
    let mut camera = match camera_query.get_single_mut() {
        Ok(c) => c,
        Err(_) => return,
    };

    for event in focus_events.read() {
        let target_part = event.0;
        
        // Find the bounding box of the target part
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        let mut found = false;

        for (part, transform, mesh_handle) in actor_query.iter() {
            if *part == target_part {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    if let Some(aabb) = mesh.compute_aabb() {
                        let center = Vec3::from(aabb.center);
                        let half_extents = Vec3::from(aabb.half_extents);
                        
                        let world_center = transform.transform_point(center);
                        // Simplified AABB expansion in world space
                        let world_half_extents = transform.to_scale_rotation_translation().0 * half_extents;
                        
                        min = min.min(world_center - world_half_extents);
                        max = max.max(world_center + world_half_extents);
                        found = true;
                    }
                }
            }
        }

        if found {
            let center = (min + max) / 2.0;
            let size = (max - min).length();
            
            camera.target_focus = center;
            camera.target_radius = size * 1.5;
        }
    }
}

pub fn inspection_highlight_system(
    settings: Res<InspectionSettings>,
    mut part_query: Query<(&ActorPart, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Only update if settings changed
    if !settings.is_changed() { return; }

    for (part, mat_handle) in part_query.iter_mut() {
        if let Some(mat) = materials.get_mut(mat_handle) {
            if settings.is_active && settings.hovered_part == Some(*part) {
                mat.emissive = LinearRgba::from(Color::srgb(0.3, 0.6, 1.0)) * 0.2;
            } else {
                mat.emissive = LinearRgba::BLACK;
            }
        }
    }
}

pub fn inspection_debug_draw_system(
    settings: Res<InspectionSettings>,
    mut gizmos: Gizmos,
    part_query: Query<(&ActorPart, &Handle<Mesh>, &GlobalTransform)>,
    meshes: Res<Assets<Mesh>>,
) {
    if !settings.is_active { return; }
    if !settings.show_normals && !settings.wireframe { return; }

    for (part, mesh_handle, transform) in part_query.iter() {
        // Only draw for isolated part or all if none isolated
        if let Some(isolated) = settings.isolated_part {
            if *part != isolated { continue; }
        }

        if let Some(mesh) = meshes.get(mesh_handle) {
            if settings.wireframe {
                if let Some(aabb) = mesh.compute_aabb() {
                    gizmos.cuboid(*transform * Transform::from_translation(Vec3::from(aabb.center)).with_scale(Vec3::from(aabb.half_extents) * 2.0), Color::srgba(1.0, 1.0, 1.0, 0.2));
                }
            }

            if settings.show_normals {
                if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    if let Some(normals) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
                        let pos_values = positions.as_float3().unwrap();
                        let norm_values = normals.as_float3().unwrap();
                        
                        for (i, (p, n)) in pos_values.iter().zip(norm_values.iter()).enumerate() {
                            if i % 100 != 0 { continue; }
                            let start = transform.transform_point(Vec3::from(*p));
                            let end = start + transform.to_scale_rotation_translation().1 * Vec3::from(*n) * 0.05;
                            gizmos.line(start, end, Color::srgb(0.3, 1.0, 0.3));
                        }
                    }
                }
            }
        }
    }
}

pub fn inspection_ui_logic_system(
    mut settings: ResMut<InspectionSettings>,
    mut focus_events: EventWriter<InspectionFocusEvent>,
    focus_query: Query<(&Interaction, &PartFocusButton), (Changed<Interaction>, With<PartFocusButton>)>,
    solo_query: Query<(&Interaction, &PartSoloButton), (Changed<Interaction>, With<PartSoloButton>)>,
    toggle_query: Query<(&Interaction, &InspectionToggle), (Changed<Interaction>, With<InspectionToggle>)>,
    master_query: Query<&Interaction, (Changed<Interaction>, With<InspectionMasterToggle>)>,
    hover_query: Query<(&Interaction, Option<&PartFocusButton>, Option<&PartSoloButton>)>,
    mut btn_query: Query<(&mut BackgroundColor, Option<&PartSoloButton>, Option<&InspectionToggle>, Option<&InspectionMasterToggle>, Option<&Interaction>)>,
) {
    // Update hovered_part
    settings.hovered_part = None;
    for (interaction, focus_opt, solo_opt) in hover_query.iter() {
        if *interaction == Interaction::Hovered {
            if let Some(focus) = focus_opt { settings.hovered_part = Some(focus.0); }
            if let Some(solo) = solo_opt { settings.hovered_part = Some(solo.0); }
        }
    }

    // Handle Master Toggle
    for interaction in master_query.iter() {
        if *interaction == Interaction::Pressed {
            settings.is_active = !settings.is_active;
        }
    }

    // Handle Focus Buttons
    for (interaction, focus_btn) in focus_query.iter() {
        if *interaction == Interaction::Pressed {
            settings.is_active = true;
            focus_events.send(InspectionFocusEvent(focus_btn.0));
        }
    }

    // Handle Solo Buttons
    for (interaction, solo_btn) in solo_query.iter() {
        if *interaction == Interaction::Pressed {
            settings.is_active = true;
            if settings.isolated_part == Some(solo_btn.0) {
                settings.isolated_part = None;
            } else {
                settings.isolated_part = Some(solo_btn.0);
            }
        }
    }

    // Handle Toggle Buttons
    for (interaction, toggle) in toggle_query.iter() {
        if *interaction == Interaction::Pressed {
            settings.is_active = true;
            match toggle.0 {
                InspectionToggleType::Ghost => settings.ghost_mode = !settings.ghost_mode,
                InspectionToggleType::Wireframe => settings.wireframe = !settings.wireframe,
                InspectionToggleType::Normals => settings.show_normals = !settings.show_normals,
            }
        }
    }

    // Update Button Backgrounds to show active state
    for (mut bg, solo_opt, toggle_opt, master_opt, interaction_opt) in btn_query.iter_mut() {
        let active = if let Some(solo) = solo_opt {
            settings.is_active && settings.isolated_part == Some(solo.0)
        } else if let Some(toggle) = toggle_opt {
            settings.is_active && match toggle.0 {
                InspectionToggleType::Ghost => settings.ghost_mode,
                InspectionToggleType::Wireframe => settings.wireframe,
                InspectionToggleType::Normals => settings.show_normals,
            }
        } else if master_opt.is_some() {
            settings.is_active
        } else {
            false
        };

        let hovered = interaction_opt.map_or(false, |i| *i == Interaction::Hovered);

        if active {
            *bg = Color::srgba(0.3, 0.6, 1.0, 0.6).into();
        } else if hovered {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.1).into();
        } else {
            *bg = Color::srgba(0.0, 0.0, 0.0, 0.3).into();
        }
    }
}

pub fn inspection_ui_sync_system(
    mut settings: ResMut<InspectionSettings>,
    mut last_is_active: Local<bool>,
    marker_query: Query<&Parent, With<PartsSectionMarker>>,
    mut section_query: Query<&mut CollapsibleSection>,
) {
    let Ok(parent) = marker_query.get_single() else { return; };
    let Ok(mut section) = section_query.get_mut(parent.get()) else { return; };

    let current_active = settings.is_active;
    let current_open = section.is_open;

    // 1. If section was toggled manually (by clicking the header)
    if section.is_changed() && current_open != current_active {
        settings.is_active = current_open;
        *last_is_active = current_open;
    }
    // 2. If settings.is_active was toggled (by master button, hotkey, or solo/focus buttons)
    else if current_active != *last_is_active {
        section.is_open = current_active;
        *last_is_active = current_active;
    }
    // 3. Keep last_is_active in sync even if no change was made this frame
    else {
        *last_is_active = current_active;
    }
}
