use bevy::prelude::*;
use super::super::{MainEditorCamera, GizmoCamera, SlicingSettings, ViewportSettings, SlicingContours, ActorBounds, SlicingGizmoType, SlicingGizmo, EditorHelper};

pub fn gizmo_sync_system(
    main_camera: Query<&Transform, (With<MainEditorCamera>, Without<GizmoCamera>)>,
    mut gizmo_camera: Query<&mut Transform, With<GizmoCamera>>,
) {
    if let Ok(main_transform) = main_camera.get_single() {
        if let Ok(mut gizmo_transform) = gizmo_camera.get_single_mut() {
            let distance = 3.0;
            let rotation = main_transform.rotation;
            gizmo_transform.translation = rotation * (Vec3::Z * distance);
            gizmo_transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

pub fn gizmo_viewport_system(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    viewport_settings: Res<ViewportSettings>,
    mut gizmo_camera: Query<&mut Camera, With<GizmoCamera>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Ok(mut camera) = gizmo_camera.get_single_mut() else { return; };
    
    if camera.is_active != viewport_settings.gizmos {
        camera.is_active = viewport_settings.gizmos;
    }
    
    if !camera.is_active { return; }
    
    let size = 120;
    let padding = 20;
    
    camera.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(padding, window.physical_height().saturating_sub(size + padding)),
        physical_size: UVec2::new(size, size),
        depth: 0.0..1.0,
    });
}


pub fn slicing_ui_sync_system(
    mut slicing_settings: ResMut<SlicingSettings>,
    mut range_slider_query: Query<&mut super::super::widgets::RangeSlider>,
) {
    for mut slider in range_slider_query.iter_mut() {
        if (slicing_settings.bottom_cut - slider.min_value).abs() > 0.001 ||
           (slicing_settings.top_cut - slider.max_value).abs() > 0.001 {
            
            if slider.hovered_thumb.is_none() {
                slider.min_value = slicing_settings.bottom_cut;
                slider.max_value = slicing_settings.top_cut;
            } else {
                slicing_settings.bottom_cut = slider.min_value;
                slicing_settings.top_cut = slider.max_value;
            }
        }
        
        let hovered = slider.hovered_thumb.map(|t| match t {
            super::super::widgets::RangeSliderThumb::Min => SlicingGizmoType::Bottom,
            super::super::widgets::RangeSliderThumb::Max => SlicingGizmoType::Top,
        });
        if slicing_settings.hovered_gizmo != hovered { slicing_settings.hovered_gizmo = hovered; }
    }
}

pub fn draw_slicing_contours_system(
    contour_query: Query<(&SlicingContours, &GlobalTransform)>,
    viewport_settings: Res<ViewportSettings>,
    mut gizmos: Gizmos,
) {
    if !viewport_settings.slices { return; }
    
    for (contours, transform) in contour_query.iter() {
        let matrix = transform.compute_matrix();
        for segment in &contours.segments {
            let start = matrix.transform_point3(segment[0]);
            let end = matrix.transform_point3(segment[1]);
            gizmos.line(start, end, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}

pub fn init_gizmos_system(
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let config = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.0.depth_bias = -0.01;
}

pub fn draw_actor_bounds_debug_system(
    query: Query<(&ActorBounds, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    for (bounds, transform) in query.iter() {
        let center = (bounds.max + bounds.min) / 2.0;
        let size = bounds.max - bounds.min;
        let (root_scale, root_rotation, _root_translation) = transform.to_scale_rotation_translation();
        
        gizmos.cuboid(
            Transform::from_translation(transform.transform_point(center))
                .with_scale(size * root_scale)
                .with_rotation(root_rotation),
            Color::srgba(1.0, 1.0, 1.0, 0.5)
        );
    }
}

pub fn slicing_ui_visibility_system(
    actor_query: Query<&ActorBounds>,
    mut container_query: Query<&mut Visibility, With<super::super::widgets::SlicerContainer>>,
    mut gizmo_query: Query<&mut Visibility, (With<SlicingGizmo>, Without<super::super::widgets::SlicerContainer>)>,
    viewport_settings: Res<ViewportSettings>,
) {
    let has_model = actor_query.get_single().is_ok();
    let show_slicer = has_model && viewport_settings.slices;
    let target_visibility = if show_slicer { Visibility::Visible } else { Visibility::Hidden };
    
    if let Ok(mut vis) = container_query.get_single_mut() {
        if *vis != target_visibility { *vis = target_visibility; }
    }
    for mut vis in gizmo_query.iter_mut() {
        if *vis != target_visibility { *vis = target_visibility; }
    }
}

pub fn slicing_gizmo_manager_system(
    mut commands: Commands,
    viewport_settings: Res<ViewportSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gizmo_query: Query<Entity, With<SlicingGizmo>>,
) {
    let gizmo_count = gizmo_query.iter().count();
    
    if viewport_settings.slices && gizmo_count == 0 {
        for gizmo_type in [SlicingGizmoType::Top, SlicingGizmoType::Bottom] {
            let color = match gizmo_type {
                SlicingGizmoType::Top => Color::srgba(0.3, 0.6, 1.0, 0.05),
                SlicingGizmoType::Bottom => Color::srgba(1.0, 0.6, 0.2, 0.05),
            };
            
            // Spawn Main Slicing Plane Gizmo
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::math::primitives::Circle::new(1.0))),
                    material: materials.add(StandardMaterial {
                        base_color: color,
                        alpha_mode: AlphaMode::Blend,
                        unlit: false, // Make it respect lighting and depth better
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    }),
                    transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                gizmo_type, 
                SlicingGizmo, 
                EditorHelper,
            ));
        }
    } else if !viewport_settings.slices && gizmo_count > 0 {

        for entity in gizmo_query.iter() { commands.entity(entity).despawn_recursive(); }
    }
}


pub fn slicing_gizmo_sync_system(
    slicing_settings: Res<SlicingSettings>,
    actor_query: Query<(&ActorBounds, &GlobalTransform)>,
    mut gizmo_query: Query<(&mut Transform, &SlicingGizmoType, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((bounds, transform)) = actor_query.get_single() else { return; };
    
    let height = bounds.max.y - bounds.min.y;
    let radius = (bounds.max.x - bounds.min.x).max(bounds.max.z - bounds.min.z) * 0.7;
    let world_base_y = transform.transform_point(Vec3::Y * bounds.min.y).y;

    for (mut gizmo_transform, gizmo_type, mat_handle) in gizmo_query.iter_mut() {
        let ratio = match *gizmo_type {
            SlicingGizmoType::Top => slicing_settings.top_cut,
            SlicingGizmoType::Bottom => slicing_settings.bottom_cut,
        };

        let y = world_base_y + (ratio * height);
        let pos = Vec3::new(transform.translation().x, y, transform.translation().z);
        gizmo_transform.translation = pos;
        gizmo_transform.scale = Vec3::splat(radius);
        gizmo_transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

        let is_hovered = slicing_settings.hovered_gizmo == Some(*gizmo_type);
        
        // Draw an "Always Visible" outline using Gizmos
        // The intersection effect is now achieved naturally by depth testing against the opaque model.
        // No gizmo outlines are needed as they would draw on top of everything.

        if let Some(mat) = materials.get_mut(mat_handle) {
            let alpha = if is_hovered { 0.7 } else { 0.3 }; 
            let color = match *gizmo_type {
                SlicingGizmoType::Top => Color::srgba(0.3, 0.6, 1.0, alpha),
                SlicingGizmoType::Bottom => Color::srgba(1.0, 0.6, 0.2, alpha),
            };
            mat.base_color = color;
        }
    }
}





pub fn socket_visibility_system(
    viewport_settings: Res<ViewportSettings>,
    mut socket_query: Query<&mut Visibility, With<super::super::ActorSocket>>,
) {
    let target_visibility = if viewport_settings.sockets { Visibility::Visible } else { Visibility::Hidden };
    for mut vis in socket_query.iter_mut() {
        if *vis != target_visibility {
            *vis = target_visibility;
        }
    }
}
