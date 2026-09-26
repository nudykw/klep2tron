//! State serialization (`GET /state`, `GET /ui_query`) and the types host
//! binaries use to extend or drive KTRL.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

use crate::{EditorMode, GameState, Project, Selection};

use super::ControlState;

/// Emitted by `POST /action` for the host binaries to handle.
///
/// `client_core` handles the generic lifecycle actions (`StartGame`,
/// `StartEditor`, `QuitToMenu`, `Exit`); `editor_client` adds handlers for the
/// editor-specific ones. Readers ignore actions they do not know.
#[derive(Message, Clone, Debug)]
pub struct ControlAction {
    pub name: String,
    pub args: serde_json::Value,
}

/// Extra `key: value` pairs merged into the `GET /state` response by the host
/// binary (e.g. the editor exposes the active tool and undo depth).
#[derive(Resource, Default)]
pub struct ControlExtras(pub serde_json::Map<String, serde_json::Value>);

pub(super) fn build_state(
    game_state: &Option<Res<State<GameState>>>,
    selection: &Option<Res<Selection>>,
    project: &Option<Res<Project>>,
    editor_mode: &Option<Res<EditorMode>>,
    control: &ControlState,
    diagnostics: &DiagnosticsStore,
    extras: &Option<Res<ControlExtras>>,
    transforms: &Query<(Entity, &Transform, Option<&Name>)>,
) -> String {
    use serde_json::{json, Value};

    let gs = game_state
        .as_ref()
        .map(|s| format!("{:?}", s.get()))
        .unwrap_or_else(|| "unknown".into());

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    // Cap the dump so a runaway scene cannot balloon the response.
    const ENTITY_CAP: usize = 4096;
    let collected: Vec<_> = transforms.iter().take(ENTITY_CAP + 1).collect();
    let entities: Vec<Value> = collected
        .iter()
        .take(ENTITY_CAP)
        .map(|(entity, transform, name)| {
            let t = transform.translation;
            json!({
                "entity": entity.index().index(),
                "name": name.map(|n| n.as_str().to_string()),
                "pos": [t.x, t.y, t.z],
            })
        })
        .collect();

    let selection_json = match selection {
        Some(s) => json!({"x": s.x, "z": s.z}),
        None => Value::Null,
    };
    let editor_active = editor_mode.as_ref().map(|m| m.is_active).unwrap_or(false);

    let map_json = match project {
        Some(p) if !p.rooms.is_empty() => {
            let room = &p.rooms[p.current_room_idx];
            let x_len = room.cells.len();
            let z_len = room.cells.first().map(|row| row.len()).unwrap_or(0);
            let cells: Vec<Vec<Value>> = (0..x_len)
                .map(|x| {
                    (0..z_len)
                        .map(|z| {
                            let c = &room.cells[x][z];
                            json!({"h": c.h, "tt": format!("{:?}", c.tt)})
                        })
                        .collect()
                })
                .collect();
            json!({
                "current_room": p.current_room_idx,
                "rooms": p.rooms.len(),
                "size": [x_len, z_len],
                "cells": cells,
            })
        }
        _ => Value::Null,
    };

    let mut root = json!({
        "game_state": gs,
        "editor_active": editor_active,
        "frame": control.frame,
        "paused": control.paused,
        "fps": (fps * 10.0).round() / 10.0,
        "selection": selection_json,
        "map": map_json,
        "entities": entities,
        "entities_truncated": collected.len() > ENTITY_CAP,
    });

    if let Some(extras) = extras {
        if let Some(obj) = root.as_object_mut() {
            for (key, value) in &extras.0 {
                obj.insert(key.clone(), value.clone());
            }
        }
    }

    root.to_string()
}

/// Broadcast state changes to SSE subscribers (`GET /events`).
pub(super) fn publish_state_changes(
    game_state: Res<State<GameState>>,
    editor_mode: Res<EditorMode>,
    project: Res<Project>,
    control: Res<ControlState>,
) {
    if !(game_state.is_changed() || editor_mode.is_changed() || project.is_changed()) {
        return;
    }
    let data = serde_json::json!({
        "game_state": format!("{:?}", game_state.get()),
        "editor_active": editor_mode.is_active,
        "room": project.current_room_idx,
        "rooms": project.rooms.len(),
        "frame": control.frame,
    })
    .to_string();
    super::events::publish("state", data);
}

pub(super) fn build_ui_query(
    filter: &Option<String>,
    ui_nodes: &Query<(Entity, Option<&Name>, Option<&Text>, &ComputedNode, &UiGlobalTransform)>,
    current: &std::collections::HashMap<Entity, Interaction>,
) -> String {
    use serde_json::json;

    let needle = filter.as_ref().map(|f| f.to_ascii_lowercase());
    let mut widgets = Vec::new();

    for (entity, name, text, node, transform) in ui_nodes.iter() {
        let label = name
            .map(|n| n.as_str().to_string())
            .or_else(|| text.map(|t| t.0.clone()))
            .unwrap_or_default();
        if let Some(needle) = &needle {
            if !label.to_ascii_lowercase().contains(needle) {
                continue;
            }
        }

        let scale = node.inverse_scale_factor();
        let size = node.size() * scale;
        let center = transform.translation * scale;
        widgets.push(json!({
            "entity": entity.index().index(),
            "label": label,
            "interaction": format!("{:?}", current.get(&entity).copied().unwrap_or(Interaction::None)),
            "size": [size.x, size.y],
            "center": [center.x, center.y],
            "min": [center.x - size.x / 2.0, center.y - size.y / 2.0],
            "max": [center.x + size.x / 2.0, center.y + size.y / 2.0],
        }));
        if widgets.len() >= 512 {
            break;
        }
    }

    json!({ "widgets": widgets }).to_string()
}
