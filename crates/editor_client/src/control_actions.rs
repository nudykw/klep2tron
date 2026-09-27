//! Editor-side handling of KTRL `POST /action` requests.
//!
//! Core lifecycle actions (`StartGame`, `StartEditor`, …) are handled in
//! `client_core`; this module applies the map-editing ones and publishes editor
//! state (`tool`, `undo`, `redo`) into `GET /state`.

use bevy::prelude::*;
use client_core::control::{ControlAction, ControlExtras};
use client_core::{CommandHistory, DirtyTiles, Project, Room, RoomTransition, Selection, TileType};

use crate::logic::mark_tile_dirty;
use crate::EditorState;

fn arg_usize(args: &serde_json::Value, key: &str) -> Option<usize> {
    args.get(key).and_then(|v| v.as_u64()).map(|v| v as usize)
}

fn arg_i32(args: &serde_json::Value, key: &str) -> Option<i32> {
    args.get(key).and_then(|v| v.as_i64()).map(|v| v as i32)
}

fn room_dims(project: &Project) -> (usize, usize) {
    let Some(room) = project.rooms.get(project.current_room_idx) else {
        return (0, 0);
    };
    let x_len = room.cells.len();
    let z_len = room.cells.first().map(|row| row.len()).unwrap_or(0);
    (x_len, z_len)
}

pub fn handle_editor_control_actions(
    mut actions: MessageReader<ControlAction>,
    mut project: ResMut<Project>,
    mut history: ResMut<CommandHistory>,
    mut dirty: ResMut<DirtyTiles>,
    mut selection: ResMut<Selection>,
    mut editor_state: ResMut<EditorState>,
    mut transition: ResMut<RoomTransition>,
    mut extras: Option<ResMut<ControlExtras>>,
) {
    for action in actions.read() {
        match action.name.as_str() {
            "SetSelection" => {
                let (wx, wz) = room_dims(&project);
                let x = arg_usize(&action.args, "x").unwrap_or(selection.x);
                let z = arg_usize(&action.args, "z").unwrap_or(selection.z);
                selection.x = x.min(wx.saturating_sub(1));
                selection.z = z.min(wz.saturating_sub(1));
            }
            "SetTile" => {
                if project.rooms.is_empty() {
                    continue;
                }
                let (wx, wz) = room_dims(&project);
                let x = arg_usize(&action.args, "x")
                    .unwrap_or(selection.x)
                    .min(wx.saturating_sub(1));
                let z = arg_usize(&action.args, "z")
                    .unwrap_or(selection.z)
                    .min(wz.saturating_sub(1));
                let h = arg_i32(&action.args, "h");
                let tt = action
                    .args
                    .get("tt")
                    .and_then(|v| v.as_str())
                    .and_then(TileType::parse);
                history.push_undo(&project);
                let room_idx = project.current_room_idx;
                let cell = &mut project.rooms[room_idx].cells[x][z];
                if let Some(tt) = tt {
                    cell.tt = tt;
                }
                if let Some(h) = h {
                    cell.h = h;
                }
                if cell.h < 0 {
                    cell.tt = TileType::Empty;
                } else if cell.tt == TileType::Empty {
                    cell.tt = editor_state.current_type;
                    if cell.h == 0 {
                        cell.h = 1;
                    }
                }
                mark_tile_dirty(x, z, &mut dirty);
            }
            "SetTileType" => {
                if let Some(tt) = action
                    .args
                    .get("tt")
                    .and_then(|v| v.as_str())
                    .and_then(TileType::parse)
                {
                    editor_state.current_type = tt;
                }
            }
            "Undo" => {
                if let Some(prev) = history.undo(&project) {
                    *project = prev;
                    dirty.full_rebuild = true;
                }
            }
            "Redo" => {
                if let Some(next) = history.redo(&project) {
                    *project = next;
                    dirty.full_rebuild = true;
                }
            }
            "NextRoom" => {
                history.push_undo(&project);
                let next = project.current_room_idx + 1;
                if next >= project.rooms.len() {
                    project.rooms.push(Room::default());
                }
                transition.start(next);
            }
            "PrevRoom" => {
                if project.current_room_idx > 0 {
                    history.push_undo(&project);
                    transition.start(project.current_room_idx - 1);
                }
            }
            "AddRoom" => {
                history.push_undo(&project);
                project.rooms.push(Room::default());
            }
            "ClearRoom" => {
                history.push_undo(&project);
                let room_idx = project.current_room_idx;
                if let Some(room) = project.rooms.get_mut(room_idx) {
                    *room = Room::default();
                }
                dirty.full_rebuild = true;
            }
            "SaveMap" => {
                if let Ok(json) = serde_json::to_string_pretty(&*project) {
                    let _ = std::fs::write("assets/map.json", json);
                }
            }
            "LoadMap" => {
                if let Ok(content) = std::fs::read_to_string("assets/map.json") {
                    if let Ok(loaded) = serde_json::from_str::<Project>(&content) {
                        *project = loaded;
                        dirty.full_rebuild = true;
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(extras) = extras.as_mut() {
        extras.0.insert(
            "tool".into(),
            serde_json::json!(format!("{:?}", editor_state.current_type)),
        );
        extras.0.insert("undo".into(), serde_json::json!(history.undo_stack.len()));
        extras.0.insert("redo".into(), serde_json::json!(history.redo_stack.len()));
    }
}
