use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::{HudText, Selection, Project};

pub fn hud_update_system(
    diagnostics: Res<DiagnosticsStore>, 
    project: Res<Project>, 
    selection: Option<Res<Selection>>, 
    mut query: Query<&mut Text, With<HudText>>
) {
    let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d| d.smoothed()).unwrap_or(0.0);
    for mut text in query.iter_mut() {
        if let Some(sel) = selection.as_ref() {
            if project.current_room_idx < project.rooms.len() {
                let room = &project.rooms[project.current_room_idx];
                let cell = room.cells[sel.x][sel.z];
                text.sections[0].value = format!("FPS: {:.0} | ROOM: {} | POS: {}, {}, {}", fps, project.current_room_idx, sel.x, sel.z, cell.h);
            }
        } else {
            text.sections[0].value = format!("FPS: {:.0} | ROOM: {}", fps, project.current_room_idx);
        }
    }
}
