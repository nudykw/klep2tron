use bevy::prelude::*;
use crate::actor_editor::{
    ScalingSettings, ActorBounds, Actor3DRoot, EditorStatus, SlicingSettings,
    widgets::TextInput,
    ui::inspector::types::*,
};

pub fn mesh_scaling_ui_sync_system(
    mut scaling_settings: ResMut<ScalingSettings>,
    mut input_query: Query<&mut TextInput>,
    marker_x: Query<Entity, With<ScalingInputX>>,
    marker_y: Query<Entity, With<ScalingInputY>>,
    marker_z: Query<Entity, With<ScalingInputZ>>,
    mut btn_query: Query<(&Interaction, &mut BackgroundColor), With<ScalingLinkToggle>>,
) {
    let mut x_val = None;
    let mut y_val = None;
    let mut z_val = None;

    for entity in marker_x.iter() {
        if let Ok(input) = input_query.get(entity) {
            if input.is_focused {
                if let Ok(val) = input.value.parse::<f32>() {
                    x_val = Some(val);
                }
            }
        }
    }
    for entity in marker_y.iter() {
        if let Ok(input) = input_query.get(entity) {
            if input.is_focused {
                if let Ok(val) = input.value.parse::<f32>() {
                    y_val = Some(val);
                }
            }
        }
    }
    for entity in marker_z.iter() {
        if let Ok(input) = input_query.get(entity) {
            if input.is_focused {
                if let Ok(val) = input.value.parse::<f32>() {
                    z_val = Some(val);
                }
            }
        }
    }

    if let Some(val) = x_val {
        if (scaling_settings.width - val).abs() > 0.0001 {
            if scaling_settings.link_proportions && scaling_settings.width.abs() > 0.0001 {
                let ratio = val / scaling_settings.width;
                scaling_settings.height *= ratio;
                scaling_settings.length *= ratio;
                info!("Proportional Sync from X: val={}, ratio={}", val, ratio);
            }
            scaling_settings.width = val;
        }
    } else if let Some(val) = y_val {
        if (scaling_settings.height - val).abs() > 0.0001 {
            if scaling_settings.link_proportions && scaling_settings.height.abs() > 0.0001 {
                let ratio = val / scaling_settings.height;
                scaling_settings.width *= ratio;
                scaling_settings.length *= ratio;
                info!("Proportional Sync from Y: val={}, ratio={}", val, ratio);
            }
            scaling_settings.height = val;
        }
    } else if let Some(val) = z_val {
        if (scaling_settings.length - val).abs() > 0.0001 {
            if scaling_settings.link_proportions && scaling_settings.length.abs() > 0.0001 {
                let ratio = val / scaling_settings.length;
                scaling_settings.width *= ratio;
                scaling_settings.height *= ratio;
                info!("Proportional Sync from Z: val={}, ratio={}", val, ratio);
            }
            scaling_settings.length = val;
        }
    }

    // 3. Sync from Resource to UI (when not focused)
    for entity in marker_x.iter() {
        if let Ok(mut input) = input_query.get_mut(entity) {
            if !input.is_focused {
                let s = format!("{:.2}", scaling_settings.width);
                if input.value != s { input.value = s; }
            }
        }
    }
    for entity in marker_y.iter() {
        if let Ok(mut input) = input_query.get_mut(entity) {
            if !input.is_focused {
                let s = format!("{:.2}", scaling_settings.height);
                if input.value != s { input.value = s; }
            }
        }
    }
    for entity in marker_z.iter() {
        if let Ok(mut input) = input_query.get_mut(entity) {
            if !input.is_focused {
                let s = format!("{:.2}", scaling_settings.length);
                if input.value != s { input.value = s; }
            }
        }
    }

    // 3. Handle Link Toggle
    for (interaction, mut bg) in btn_query.iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            // Background color is handled below
        }
        
        if scaling_settings.link_proportions {
            *bg = Color::srgba(0.3, 0.6, 1.0, 0.6).into();
        } else {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
        }
    }
}

pub fn mesh_scaling_interaction_system(
    mut scaling_settings: ResMut<ScalingSettings>,
    btn_query: Query<&Interaction, (With<ScalingLinkToggle>, Changed<Interaction>)>,
) {
    for interaction in btn_query.iter() {
        if *interaction == Interaction::Pressed {
            scaling_settings.link_proportions = !scaling_settings.link_proportions;
            info!("Scaling Link Proportions: {}", scaling_settings.link_proportions);
        }
    }
}

pub fn mesh_scaling_apply_system(
    mut actor_query: Query<(Entity, &ActorBounds, &mut Transform), With<Actor3DRoot>>,
    scaling_settings: Res<ScalingSettings>,
    btn_query: Query<&Interaction, (With<ScalingApplyButton>, Changed<Interaction>)>,
    status: Res<EditorStatus>,
    mut slicing_settings: ResMut<SlicingSettings>,
    mut action_stack: ResMut<crate::actor_editor::systems::undo_redo::ActionStack>,
) {
    let Ok((entity, bounds, mut transform)) = actor_query.get_single_mut() else { return; };
    
    for interaction in btn_query.iter() {
        if *interaction == Interaction::Pressed && *status == EditorStatus::Ready {
            let size = bounds.max - bounds.min;
            if size.x > 0.0 && size.y > 0.0 && size.z > 0.0 {
                let new_scale = Vec3::new(
                    scaling_settings.width / size.x,
                    scaling_settings.height / size.y,
                    scaling_settings.length / size.z,
                );
                
                if (transform.scale - new_scale).length() > 0.0001 {
                    let old_scale = transform.scale;
                    
                    action_stack.push(Box::new(crate::actor_editor::systems::undo_redo::ScaleModelCommand {
                        entity,
                        old_scale,
                        new_scale,
                    }));
                    
                    transform.scale = new_scale;
                    slicing_settings.trigger_slice = true; // Refresh gizmos and potentially re-slice
                    info!("Applied new scale: {:?}", new_scale);
                }
            }
        }
    }
}
