use bevy::prelude::*;
use bevy::render::mesh::Indices;
use super::super::{
    ActorPart, SelectedTriangles, TargetPart, TargetPartButton, SlicingSettings, CapTriangleRange,
};
use super::undo_redo::{ActionStack, GeometryReassignCommand};

/// Переносит выделенные треугольники в целевую часть при нажатии кнопки Target Part.
pub fn geometry_reassign_system(
    btn_query: Query<(&Interaction, &TargetPartButton), Changed<Interaction>>,
    mut target_part: ResMut<TargetPart>,
    mut part_query: Query<(Entity, &ActorPart, &Handle<Mesh>, &mut SelectedTriangles)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut action_stack: ResMut<ActionStack>,
    slicing_settings: Res<SlicingSettings>,
) {
    if !slicing_settings.manual_mode { return; }

    // Определяем нажатую кнопку
    let mut pressed_part: Option<ActorPart> = None;
    for (interaction, btn) in btn_query.iter() {
        if *interaction == Interaction::Pressed {
            pressed_part = Some(btn.0);
            *target_part = match btn.0 {
                ActorPart::Head   => TargetPart::Head,
                ActorPart::Body   => TargetPart::Body,
                ActorPart::Engine => TargetPart::Engine,
            };
        }
    }
    let Some(dst_part_type) = pressed_part else { return; };

    // Собираем данные обо всех частях (без mut borrow)
    let parts_data: Vec<(Entity, ActorPart, Handle<Mesh>)> = part_query
        .iter()
        .map(|(e, p, h, _)| (e, *p, h.clone()))
        .collect();

    // Находим dst entity и handle
    let dst_info = parts_data.iter().find(|(_, p, _)| *p == dst_part_type).cloned();
    let Some((dst_entity, _, dst_handle)) = dst_info else { return; };

    // Для каждой части-источника с непустым выделением — переносим
    for (src_entity, src_part, src_handle) in &parts_data {
        if *src_part == dst_part_type { continue; }

        // Получаем индексы выделения
        let selected_indices = {
            let Ok((_, _, _, sel)) = part_query.get(*src_entity) else { continue; };
            if sel.indices.is_empty() { continue; }
            sel.indices.clone()
        };

        // Клонируем старые меши для Undo
        let old_src = match meshes.get(src_handle) {
            Some(m) => m.clone(),
            None => continue,
        };
        let old_dst = match meshes.get(&dst_handle) {
            Some(m) => m.clone(),
            None => continue,
        };

        // Переносим геометрию
        let (new_src, new_dst) = super::super::geometry::transfer_triangles(
            &old_src,
            &old_dst,
            &selected_indices,
        );

        // Применяем немедленно
        if let Some(m) = meshes.get_mut(src_handle) { *m = new_src.clone(); }
        if let Some(m) = meshes.get_mut(&dst_handle) { *m = new_dst.clone(); }

        // Очищаем выделение на src
        if let Ok((_, _, _, mut sel)) = part_query.get_mut(*src_entity) {
            sel.indices.clear();
        }

        // Регистрируем в Undo/Redo
        action_stack.push(Box::new(GeometryReassignCommand {
            src_entity: *src_entity,
            dst_entity,
            old_src_mesh: old_src,
            old_dst_mesh: old_dst,
            new_src_mesh: new_src,
            new_dst_mesh: new_dst,
        }));

        info!("Geometry Reassign: {:?} -> {:?}, {} triangles",
            src_part, dst_part_type, selected_indices.len());
    }
}

/// Удаляет процедурные крышки из мешей частей при переходе Auto → Manual с Keep Caps = OFF.
pub fn caps_cleanup_system(
    mut slicing_settings: ResMut<SlicingSettings>,
    part_query: Query<(&Handle<Mesh>, &CapTriangleRange)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !slicing_settings.trigger_caps_cleanup { return; }
    slicing_settings.trigger_caps_cleanup = false;

    for (handle, cap_range) in part_query.iter() {
        if let Some(mesh) = meshes.get_mut(handle) {
            if let Some(indices) = mesh.indices_mut() {
                match indices {
                    Indices::U32(ref mut idx) => {
                        let keep = cap_range.cap_start_tri * 3;
                        if keep < idx.len() {
                            idx.truncate(keep);
                            info!("CapCleanup: truncated to {} indices (cap_start_tri={})",
                                keep, cap_range.cap_start_tri);
                        }
                    }
                    Indices::U16(ref mut idx) => {
                        let keep = cap_range.cap_start_tri * 3;
                        if keep < idx.len() { idx.truncate(keep); }
                    }
                }
            }
        }
    }
}

/// Синхронизирует визуальное состояние кнопок Target Part (подсветка активной).
pub fn target_part_button_visual_system(
    target_part: Res<TargetPart>,
    mut btn_query: Query<(&mut BackgroundColor, &mut BorderColor, &Interaction, &TargetPartButton)>,
) {
    if !target_part.is_changed() && !btn_query.is_empty() {
        // Всё равно обновляем при hover — Changed<Interaction> не нужен здесь
    }
    for (mut bg, mut border, interaction, btn) in btn_query.iter_mut() {
        let is_active = matches!(
            (*target_part, btn.0),
            (TargetPart::Head, ActorPart::Head)
            | (TargetPart::Body, ActorPart::Body)
            | (TargetPart::Engine, ActorPart::Engine)
        );

        let base = if is_active {
            Color::srgba(0.3, 0.6, 1.0, 0.4)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.05)
        };

        let final_color = if *interaction == Interaction::Hovered {
            match base {
                Color::Srgba(c) => Color::Srgba(bevy::color::Srgba { alpha: (c.alpha * 1.5).min(1.0), ..c }),
                _ => base,
            }
        } else {
            base
        };

        *bg = final_color.into();
        *border = if is_active {
            Color::srgba(0.3, 0.6, 1.0, 0.8).into()
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.1).into()
        };
    }
}
