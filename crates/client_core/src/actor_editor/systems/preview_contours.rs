use bevy::prelude::*;
use super::super::{
    SlicingSettings, ActorBounds, OriginalMeshComponent, PreviewContours,
    geometry,
};

pub fn preview_contours_system(
    mut commands: Commands,
    slicing_settings: Res<SlicingSettings>,
    actor_root_query: Query<(Entity, &ActorBounds, &GlobalTransform), With<crate::actor_editor::Actor3DRoot>>,
    mesh_query: Query<(&OriginalMeshComponent, &GlobalTransform)>,
    meshes: Res<Assets<Mesh>>,
    mut preview_query: Query<&mut PreviewContours>,
) {
    if slicing_settings.dragging_gizmo.is_none() {
        for _entity in preview_query.iter().map(|_| ()).collect::<Vec<_>>() {
            if let Ok((root_entity, _, _)) = actor_root_query.get_single() {
                commands.entity(root_entity).remove::<PreviewContours>();
            }
        }
        return;
    }
    
    let Ok((root_entity, bounds, root_global)) = actor_root_query.get_single() else {
        return;
    };
    
    let local_height = bounds.max.y - bounds.min.y;
    let plane_top_local = bounds.min.y + slicing_settings.top_cut * local_height;
    let plane_bottom_local = bounds.min.y + slicing_settings.bottom_cut * local_height;
    
    let mut all_segments = Vec::new();
    
    for (original, transform) in mesh_query.iter() {
        if let Some(mesh) = meshes.get(&original.0) {
            let start_time = std::time::Instant::now();
            
            let world_top = root_global.compute_matrix().transform_point3(Vec3::new(0.0, plane_top_local, 0.0));
            let world_bottom = root_global.compute_matrix().transform_point3(Vec3::new(0.0, plane_bottom_local, 0.0));
            
            let local_matrix = transform.compute_matrix();
            let inv_local = local_matrix.inverse();
            
            let mesh_local_top = inv_local.transform_point3(world_top).y;
            let mesh_local_bottom = inv_local.transform_point3(world_bottom).y;
            
            let segments = geometry::contour_calculator::calculate_contours_only(
                mesh,
                mesh_local_top,
                mesh_local_bottom,
            );
            
            info!("Preview Contours: {:?} ({} segments)", start_time.elapsed(), segments.len());
            
            for segment in segments {
                let world_start = local_matrix.transform_point3(segment[0]);
                let world_end = local_matrix.transform_point3(segment[1]);
                all_segments.push([world_start, world_end]);
            }
        }
    }
    
    if let Ok(mut preview) = preview_query.get_mut(root_entity) {
        preview.segments = all_segments;
    } else {
        commands.entity(root_entity).insert(PreviewContours {
            segments: all_segments,
            is_preview: true,
        });
    }
}
