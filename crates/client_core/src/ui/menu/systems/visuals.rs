use bevy::prelude::*;
use crate::GraphicsSettings;
use super::super::types::*;

pub fn menu_navigation_system(
    mut commands: Commands,
    container_query: Query<(Entity, &MenuContainer), Changed<MenuContainer>>,
    item_query: Query<(Entity, &MenuItem, &Parent)>,
    children_query: Query<&Children>,
) {
    for (container_entity, container) in container_query.iter() {
        let mut descendants = Vec::new();
        let mut stack = vec![container_entity];
        while let Some(current) = stack.pop() {
            if let Ok(children) = children_query.get(current) {
                for child in children.iter() {
                    descendants.push(*child);
                    stack.push(*child);
                }
            }
        }

        for (entity, item, _parent) in item_query.iter() {
            if !descendants.contains(&entity) { continue; }

            if let Some(mut e) = commands.get_entity(entity) {
                if item.index == container.current_selection {
                    e.insert(MenuFocus);
                } else {
                    e.remove::<MenuFocus>();
                }
            }
        }
    }
}

pub fn menu_scrolling_system(
    container_query: Query<&MenuContainer, With<MenuItemRoot>>,
    mut scroll_query: Query<(Entity, &mut Style), With<MenuScrollContainer>>,
    mut commands: Commands,
) {
    let Ok(container) = container_query.get_single() else { return; };
    for (entity, mut style) in scroll_query.iter_mut() {
        if commands.get_entity(entity).is_none() { continue; }
        
        let item_height = 60.0; 
        let viewport_height = 420.0;
        let target_offset = (viewport_height / 2.0) - (container.current_selection as f32 * item_height) - (item_height / 2.0);
        
        style.top = Val::Px(target_offset);
    }
}

pub fn tooltip_system(
    _commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MenuTooltip, &mut Visibility)>,
) {
    for (_entity, mut tooltip, mut visibility) in query.iter_mut() {
        tooltip.timer.tick(time.delta());
        if tooltip.timer.finished() {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn menu_tooltip_system(
    focus_query: Query<&MenuItem, With<MenuFocus>>,
    mut tooltip_query: Query<&mut Text, With<TooltipDisplay>>,
) {
    let mut text_val = "".to_string();
    if let Ok(item) = focus_query.get_single() {
        if let Some(tooltip) = &item.tooltip {
            text_val = tooltip.clone();
        }
    }
    
    for mut text in tooltip_query.iter_mut() {
        if text.sections[0].value != text_val {
            text.sections[0].value = text_val.clone();
        }
    }
}

pub fn input_hint_system(
    input_device: Res<InputDevice>,
    mut query: Query<&mut Text, With<InputHintFooter>>,
) {
    for mut text in query.iter_mut() {
        let hint = match *input_device {
            InputDevice::Keyboard => "[Arrows] Navigate/Change  [Enter] Select  [Esc] Back",
            InputDevice::Gamepad => "(D-Pad) Navigate/Change  (A) Select  (B) Back",
            InputDevice::Mouse => "Hover to Focus  Click to Cycle/Select",
            InputDevice::Touch => "Tap to Select  Long Press for Hint",
        };
        text.sections[0].value = hint.to_string();
    }
}

pub fn menu_visual_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &MenuItem, &mut BackgroundColor, &mut BorderColor, &mut Transform, Option<&MenuFocus>)>,
    settings: Res<GraphicsSettings>,
    pending: Res<PendingGraphicsSettings>,
) {
    let has_changes = **pending != *settings;
    let t = (time.elapsed_seconds() * 3.0).sin() * 0.5 + 0.5;

    for (entity, item, mut bg, mut border, mut transform, focus) in query.iter_mut() {
        if commands.get_entity(entity).is_none() { continue; }

        let is_apply = item.action == MenuAction::ApplySettings;
        let is_dimmed = item.is_disabled || (is_apply && !has_changes);

        if is_dimmed {
            *bg = Color::srgba(0.1, 0.1, 0.1, 0.2).into();
            *border = Color::NONE.into();
            transform.scale = Vec3::splat(1.0);
        } else if focus.is_some() {
            let bg_alpha = 0.15 + t * 0.15;
            let border_alpha = 0.4 + t * 0.4;
            *bg = Color::srgba(0.3, 0.6, 1.0, bg_alpha).into();
            *border = Color::srgba(0.4, 0.7, 1.0, border_alpha).into();
            transform.scale = Vec3::splat(1.05);
        } else {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
            *border = Color::NONE.into();
            transform.scale = Vec3::splat(1.0);
        }
    }
}
