use bevy::prelude::*;
use super::super::{SlicingSettings, ActorBounds, ActorEditorEntity, OriginalMeshComponent, SlicingContours, ActorPart, geometry, ImportProgress, EditorStatus, EditorHelper};

#[derive(Resource, Default)]
pub struct SlicingTask(pub Option<bevy::tasks::Task<SlicingResult>>);

pub struct SlicingResult {
    pub mesh_parts: Vec<(Entity, geometry::SlicedParts)>,
}

pub fn mesh_slicing_system(
    mut commands: Commands,
    mut slicing_settings: ResMut<SlicingSettings>,
    actor_root_query: Query<(&ActorBounds, &GlobalTransform), With<ActorEditorEntity>>,
    mesh_query: Query<(Entity, &OriginalMeshComponent, &GlobalTransform, Option<&Handle<StandardMaterial>>, Option<&mut SlicingContours>)>,
    child_query: Query<Entity, With<ActorPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut slicing_task: ResMut<SlicingTask>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut progress: ResMut<ImportProgress>,
    mut status: ResMut<EditorStatus>,
) {
    // 1. Check if a task is already running
    if let Some(ref mut task) = slicing_task.0 {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            // Apply result
            for (parent_entity, parts) in result.mesh_parts {
                let mut spawn_part = |cmds: &mut ChildBuilder, mesh_opt: Option<Mesh>, name: &str, part_type: ActorPart, color: Color| {
                    if let Some(m) = mesh_opt {
                        cmds.spawn((
                            PbrBundle {
                                mesh: meshes.add(m),
                                material: materials.add(StandardMaterial {
                                    base_color: color,
                                    perceptual_roughness: 0.5,
                                    ..default()
                                }),
                                visibility: Visibility::Visible,
                                ..default()
                            },
                            EditorHelper,
                            part_type,
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
    let Ok((bounds, _)) = actor_root_query.get_single() else { return; };
    
    // Check if we need to apply initial slice (only after auto-setup)
    let needs_initial_slice = child_query.is_empty() && !mesh_query.is_empty();
    
    // Check if values actually changed
    let values_changed = (slicing_settings.top_cut - slicing_settings.last_top).abs() > 0.001 ||
                         (slicing_settings.bottom_cut - slicing_settings.last_bottom).abs() > 0.001;

    // Trigger ONLY on mouse release IF values actually changed, or initial load
    let trigger = mouse_button.just_released(MouseButton::Left) || needs_initial_slice;

    if !trigger || (!values_changed && !needs_initial_slice) { return; }
    
    // Update last values to prevent re-triggering
    slicing_settings.last_top = slicing_settings.top_cut;
    slicing_settings.last_bottom = slicing_settings.bottom_cut;

    
    // Despawn old parts immediately to show we are working
    for child in child_query.iter() { commands.entity(child).despawn_recursive(); }

    // Use LOCAL coordinates for slicing to avoid rotation issues
    // Apply defaults for the first run BEFORE calculating planes
    if needs_initial_slice {
        slicing_settings.top_cut = 0.75;
        slicing_settings.bottom_cut = 0.25;
        info!("Applied auto-slicing defaults (0.75 / 0.25)");
    }

    let local_height = bounds.max.y - bounds.min.y;
    let plane_top_local = bounds.min.y + slicing_settings.top_cut * local_height;
    let plane_bottom_local = bounds.min.y + slicing_settings.bottom_cut * local_height;

    info!("Slicing (Local): top={}, bottom={}", plane_top_local, plane_bottom_local);


    // Capture data for thread
    let mut mesh_data = Vec::new();
    for (entity, original, transform, _, contours_opt) in mesh_query.iter() {
        if let Some(mesh) = meshes.get(&original.0) {
            // Collect data if we have the component OR if it's the very first slice
            if contours_opt.is_some() || needs_initial_slice {
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

    let task = thread_pool.spawn(async move {
        let mut results = Vec::new();
        for (entity, mesh, inv_local) in mesh_data {
            let mesh_local_top = inv_local.transform_point3(Vec3::new(0.0, plane_top_local, 0.0)).y;
            let mesh_local_bottom = inv_local.transform_point3(Vec3::new(0.0, plane_bottom_local, 0.0)).y;

            let parts = geometry::slicer::split_mesh_by_planes(&mesh, mesh_local_top, mesh_local_bottom);
            results.push((entity, parts));
        }
        SlicingResult { mesh_parts: results }
    });

    slicing_task.0 = Some(task);
    info!("Started async slicing task...");
}

