use bevy::prelude::*;
use client_core::{Project, Selection, DirtyTiles, CommandHistory, HelpState};
use crate::{EditorState, TileTypeButton, HelpButton, TooltipText, TooltipUi, mark_tile_dirty};

pub fn editor_ui_system(
    mut interaction_query: Query<(&Interaction, Option<&TileTypeButton>, Option<&HelpButton>, &mut BackgroundColor, &mut BorderColor)>,
    mut editor_state: ResMut<EditorState>,
    mut project: ResMut<Project>,
    mut history: ResMut<CommandHistory>,
    mut help_state: ResMut<HelpState>,
    selection: Res<Selection>,
    mut dirty: ResMut<DirtyTiles>,
) {
    for (interaction, tt_btn, help_btn, mut color, mut border) in interaction_query.iter_mut() {
        let is_selected = tt_btn.map_or(false, |b| editor_state.current_type == b.0);
        match *interaction {
            Interaction::Pressed => {
                if let Some(tt_btn) = tt_btn {
                    let mut changed = false;
                    if !is_selected { editor_state.current_type = tt_btn.0; changed = true; }
                    let room_idx = project.current_room_idx;
                    let cell = project.rooms[room_idx].cells[selection.x][selection.z];
                    if cell.h >= 0 && cell.tt != tt_btn.0 { 
                        history.push_undo(&project);
                        let cell = &mut project.rooms[room_idx].cells[selection.x][selection.z];
                        cell.tt = tt_btn.0; 
                        changed = true; 
                        mark_tile_dirty(selection.x, selection.z, &mut dirty);
                    }
                    if changed { *color = Color::srgb(0.0, 1.0, 1.0).into(); }
                }
                if help_btn.is_some() {
                    help_state.is_open = !help_state.is_open;
                }
            }
            Interaction::Hovered => {
                if is_selected { *color = Color::srgb(0.0, 0.8, 0.8).into(); }
                else if help_btn.is_some() { *color = Color::srgb(0.0, 0.5, 0.5).into(); }
                else { *color = Color::srgb(0.4, 0.4, 0.4).into(); }
            }
            Interaction::None => {
                if is_selected { 
                    *color = Color::srgb(0.0, 0.6, 0.6).into(); 
                    *border = Color::srgb(1.0, 1.0, 1.0).into();
                } else if help_btn.is_some() {
                    *color = Color::srgb(0.1, 0.3, 0.3).into();
                    *border = Color::srgb(0.0, 0.8, 0.8).into();
                } else { 
                    *color = Color::srgb(0.2, 0.2, 0.2).into(); 
                    *border = Color::srgb(0.4, 0.4, 0.4).into();
                }
            }
        }
    }
}

pub fn editor_tooltip_system(
    windows: Query<&Window>,
    mut tooltip_query: Query<(&mut Style, &mut Visibility, &Children), With<TooltipUi>>,
    mut text_query: Query<&mut Text>,
    interaction_query: Query<(&Interaction, &TooltipText)>,
) {
    let window = windows.single();
    let mut tooltip_active = false;

    if let Ok((mut style, mut visibility, children)) = tooltip_query.get_single_mut() {
        for (interaction, tooltip) in interaction_query.iter() {
            if *interaction == Interaction::Hovered {
                tooltip_active = true;
                if let Some(mut text) = text_query.get_mut(children[0]).ok() {
                    text.sections[0].value = tooltip.0.clone();
                }
                
                if let Some(cursor_pos) = window.cursor_position() {
                    style.left = Val::Px(cursor_pos.x + 15.0);
                    style.top = Val::Px(cursor_pos.y + 15.0);
                    style.display = Display::Flex;
                    *visibility = Visibility::Visible;
                }
                break;
            }
        }

        if !tooltip_active {
            style.display = Display::None;
            *visibility = Visibility::Hidden;
        }
    }
}
