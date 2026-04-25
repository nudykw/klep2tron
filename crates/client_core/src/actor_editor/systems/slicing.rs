use bevy::prelude::*;
use super::super::{SlicingSettings, ActorBounds, OriginalMeshComponent, SlicingContours, ActorPart, geometry, ImportProgress, EditorStatus, EditorHelper, systems::optimization::OptimizedMeshComponent};

#[derive(Resource, Default)]
pub struct SlicingTask(pub Option<bevy::tasks::Task<SlicingResult>>);

pub struct SlicingResult {
    pub mesh_parts: Vec<(Entity, geometry::SlicedParts)>,
}

pub fn mesh_slicing_system(
    mut commands: Commands,
    mut slicing_settings: ResMut<SlicingSettings>,
    actor_root_query: Query<(&ActorBounds, &GlobalTransform), With<crate::actor_editor::Actor3DRoot>>,
    mesh_query: Query<(Entity, &OriginalMeshComponent, Option<&OptimizedMeshComponent>, &GlobalTransform, Option<&Handle<StandardMaterial>>, Option<&mut SlicingContours>)>,
    child_query: Query<(Entity, &ActorPart, &Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut slicing_task: ResMut<SlicingTask>,
    _mouse_button: Res<ButtonInput<MouseButton>>,

    mut progress: ResMut<ImportProgress>,
    mut status: ResMut<EditorStatus>,
    mut action_stack: ResMut<crate::actor_editor::systems::undo_redo::ActionStack>,
    opt_settings: Res<super::optimization::OptimizationSettings>,
) {


    // 1. Check if a task is already running
    if let Some(ref mut task) = slicing_task.0 {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            // 1. Collect current visibility states to preserve them
            let mut part_visibility = std::collections::HashMap::new();
            for (_entity, part, visibility) in child_query.iter() {
                part_visibility.insert(*part, *visibility);
            }

            // 2. Despawn old parts (Atomic swap start)
            for (entity, _, _) in child_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // 3. Apply result
            for (parent_entity, parts) in result.mesh_parts {
                let mut spawn_part = |cmds: &mut ChildBuilder, mesh_opt: Option<Mesh>, name: &str, part_type: ActorPart, color: Color| {
                    if let Some(m) = mesh_opt {
                        let visibility = part_visibility.get(&part_type).cloned().unwrap_or(Visibility::Visible);
                        
                        cmds.spawn((
                            PbrBundle {
                                mesh: meshes.add(m),
                                material: materials.add(StandardMaterial {
                                    base_color: color,
                                    perceptual_roughness: 0.5,
                                    alpha_mode: AlphaMode::Opaque,
                                    ..default()
                                }),
                                visibility,
                                ..default()
                            },
                            EditorHelper,
                            part_type,
                            bevy_mod_picking::prelude::Pickable {
                                should_block_lower: false,
                                is_hoverable: true,
                            },
                            Name::new(name.to_string()),
                        )).set_parent(parent_entity);
                    }
                };

                commands.entity(parent_entity).with_children(|p| {
                    spawn_part(p, parts.head, "Top", ActorPart::Head, Color::srgb(0.3, 0.6, 1.0));
                    spawn_part(p, parts.body, "Mid", ActorPart::Body, Color::srgb(0.8, 0.8, 0.8));
                    spawn_part(p, parts.legs, "Bottom", ActorPart::Engine, Color::srgb(1.0, 0.6, 0.2));
                });

                if parts.contours.is_empty() {
                    commands.entity(parent_entity).remove::<SlicingContours>();
                } else {
                    commands.entity(parent_entity).insert(SlicingContours { segments: parts.contours });
                }

                commands.entity(parent_entity).remove::<Handle<Mesh>>();
                commands.entity(parent_entity).remove::<Handle<StandardMaterial>>();
            }
            slicing_task.0 = None;
            info!("Async slicing completed and applied.");
            
            // Finish loading sequence if needed
            if progress.0 < 1.0 {
                progress.0 = 1.0;
                *status = EditorStatus::Ready;
            }
        }
        return; // Don't start a new task while one is running
    }

    // 2. Start new task if needed
    let Ok((bounds, root_global)) = actor_root_query.get_single() else { return; };
    
    // Check if we need to apply initial slice (only after auto-setup)
    let needs_initial_slice = child_query.is_empty() && !mesh_query.is_empty();
    
    // Check if values actually changed
    let values_changed = (slicing_settings.top_cut - slicing_settings.last_top).abs() > 0.001 ||
                         (slicing_settings.bottom_cut - slicing_settings.last_bottom).abs() > 0.001;

    let mut should_slice = needs_initial_slice;
    
    // Trigger logic for UI confirmation
    if slicing_settings.trigger_slice {
        let old_top = slicing_settings.last_top;
        let old_bottom = slicing_settings.last_bottom;
        let new_top = slicing_settings.top_cut;
        let new_bottom = slicing_settings.bottom_cut;

        if !slicing_settings.suppress_undo && old_top >= 0.0 && ((old_top - new_top).abs() > 0.001 || (old_bottom - new_bottom).abs() > 0.001) {
            action_stack.push(Box::new(crate::actor_editor::systems::undo_redo::UpdateSlicingCommand {
                old_top, old_bottom, new_top, new_bottom
            }));
        }

        should_slice = true;
        slicing_settings.trigger_slice = false;
        slicing_settings.suppress_undo = false; // Reset after trigger
        slicing_settings.needs_confirm = false;
        slicing_settings.dragging_gizmo = None;
    }

    // Trigger ONLY if values actually changed, or initial load, or explicit trigger
    let trigger = should_slice;

    if !trigger && !values_changed && !needs_initial_slice { 
        return; 
    }

    if needs_initial_slice { info!("Slicing: Initial load trigger"); }
    if slicing_settings.trigger_slice { info!("Slicing: Explicit UI trigger"); }




    
    // Update last values to prevent re-triggering (Moved after initial slice check)

    
    // Use LOCAL coordinates for slicing to avoid rotation issues
    // Apply defaults for the first run BEFORE calculating planes
    if needs_initial_slice && slicing_settings.last_top == -1.0 {
        slicing_settings.top_cut = 0.75;
        slicing_settings.bottom_cut = 0.25;
        slicing_settings.last_top = 0.75;
        slicing_settings.last_bottom = 0.25;
        info!("Applied auto-slicing defaults (0.75 / 0.25)");
    }
    
    slicing_settings.last_top = slicing_settings.top_cut;
    slicing_settings.last_bottom = slicing_settings.bottom_cut;

    let local_height = bounds.max.y - bounds.min.y;
    let plane_top_local = bounds.min.y + slicing_settings.top_cut * local_height;
    let plane_bottom_local = bounds.min.y + slicing_settings.bottom_cut * local_height;

    info!("Slicing (Local): top={}, bottom={}, caps={}, rim={}", plane_top_local, plane_bottom_local, slicing_settings.show_caps, slicing_settings.rim_thickness);


    // Capture data for thread
    let show_caps = slicing_settings.show_caps;
    let rim_thickness = slicing_settings.rim_thickness;
    let mut mesh_data = Vec::new();
    for (entity, original, optimized_opt, transform, _, _contours_opt) in mesh_query.iter() {
        let mesh_handle = if opt_settings.is_optimized {
            if let Some(opt) = optimized_opt { 
                info!("Slicing: Using OPTIMIZED mesh");
                &opt.0 
            } else {
                info!("Slicing: Waiting for OPTIMIZED mesh to appear in ECS...");
                return; 
            }
        } else {
            info!("Slicing: Using ORIGINAL mesh");
            &original.0
        };
        
        if let Some(mesh) = meshes.get(mesh_handle) {
            // Collect data if we have an explicit trigger, initial slice, or if values changed
            if trigger || needs_initial_slice || values_changed {
                let local_matrix = transform.compute_matrix();
                mesh_data.push((entity, mesh.clone(), local_matrix.inverse()));
            }
        }
    }
    
    if mesh_data.is_empty() {
        info!("No meshes found for slicing.");
        return;
    }

    let thread_pool = bevy::tasks::AsyncComputeTaskPool::get();

    let root_matrix = root_global.compute_matrix();
    let task = thread_pool.spawn(async move {
        let mut results = Vec::new();
        for (entity, mesh, inv_local) in mesh_data {
            let world_top = root_matrix.transform_point3(Vec3::new(0.0, plane_top_local, 0.0));
            let world_bottom = root_matrix.transform_point3(Vec3::new(0.0, plane_bottom_local, 0.0));

            let mesh_local_top = inv_local.transform_point3(world_top).y;
            let mesh_local_bottom = inv_local.transform_point3(world_bottom).y;

            let parts = geometry::slicer::split_mesh_by_planes(&mesh, mesh_local_top, mesh_local_bottom, show_caps, rim_thickness);
            results.push((entity, parts));
        }
        SlicingResult { mesh_parts: results }
    });

    slicing_task.0 = Some(task);
    info!("Started async slicing task...");
}

