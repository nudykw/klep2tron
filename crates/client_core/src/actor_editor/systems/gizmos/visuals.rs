use bevy::prelude::*;

pub fn xray_material_system(
    viewport_settings: Res<crate::actor_editor::ViewportSettings>,
    inspection_settings: Res<crate::actor_editor::InspectionSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    part_query: Query<(Entity, &Handle<StandardMaterial>), With<crate::actor_editor::ActorPart>>,
) {
    // If inspection is active, don't interfere with its transparency/highlighting logic
    if inspection_settings.is_active {
        return;
    }

    let current_xray = viewport_settings.xray;

    for (entity, mat_handle) in part_query.iter() {
        if let Some(material) = materials.get_mut(mat_handle) {
            let target_alpha = if current_xray { 0.1 } else { 1.0 };
            let target_mode = if current_xray { AlphaMode::Blend } else { AlphaMode::Opaque };

            // Only update if different
            if (material.base_color.alpha() - target_alpha).abs() > 0.01 || material.alpha_mode != target_mode {
                info!("Applying X-Ray to {:?}: a={}, mode={:?}", entity, target_alpha, target_mode);
                
                let mut color = material.base_color.to_srgba();
                color.alpha = target_alpha;
                
                material.base_color = Color::Srgba(color);
                material.alpha_mode = target_mode;
            }
        }
    }
}
