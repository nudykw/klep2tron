//! Parsing for `POST /batch` steps (`{"op": "…", …}`).

use serde_json::Value;

use super::http::Req;
use super::MAX_STEP_FRAMES;

fn label_of(step: &Value) -> String {
    step.get("label").and_then(|v| v.as_str()).unwrap_or_default().to_string()
}

fn id_of(step: &Value) -> u32 {
    step.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32
}

fn unsigned(step: &Value, key: &str, default: u64) -> u64 {
    step.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
}

/// `POST /mouse` body, also used as the `mouse` batch op.
pub(super) fn mouse_request(json: &Value) -> Req {
    if json.get("button").is_some() {
        Req::MouseButton {
            button: json.get("button").and_then(|v| v.as_str()).unwrap_or("left").to_string(),
            action: json.get("action").and_then(|v| v.as_str()).unwrap_or("click").to_string(),
        }
    } else {
        Req::MouseMove {
            x: json.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            y: json.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        }
    }
}

/// One `POST /batch` step.
pub(super) fn parse_batch_step(step: &Value) -> Req {
    let text = |key: &str| step.get(key).and_then(|v| v.as_str()).unwrap_or_default().to_string();
    match step.get("op").and_then(|v| v.as_str()).unwrap_or("") {
        "action" => Req::Action {
            name: text("name"),
            args: step.get("set").cloned().unwrap_or(Value::Null),
        },
        "pause" => Req::Pause { on: step.get("on").and_then(|v| v.as_bool()).unwrap_or(true) },
        "key" => Req::Key {
            key: text("key"),
            action: if text("action").is_empty() { "tap".into() } else { text("action") },
        },
        "text" => Req::Text { text: text("text") },
        "mouse" => mouse_request(step),
        "ui_click" => Req::UiClick { label: label_of(step), action: "click".into() },
        "ui_hover" => Req::UiClick { label: label_of(step), action: "hover".into() },
        "unhover" => Req::UiClick { label: String::new(), action: "unhover".into() },
        "state" => Req::State,
        "version" => Req::Version,
        "ui_query" => Req::UiQuery { label: step.get("label").map(|_| label_of(step)) },
        "logs" => Req::Logs {
            since: step.get("since").and_then(|v| v.as_u64()),
            tail: unsigned(step, "tail", 100) as usize,
            level: step.get("level").and_then(|v| v.as_str()).map(|s| s.to_string()),
        },
        "tree" | "scene_tree" => Req::SceneTree {
            root: step.get("root").and_then(|v| v.as_u64()).map(|v| v as u32),
            depth: unsigned(step, "depth", 4) as usize,
        },
        "entity" => Req::EntityDetail { id: id_of(step) },
        "mesh" => Req::MeshInfo { id: id_of(step) },
        "material" => Req::MaterialInfo { id: id_of(step) },
        "step" | "frames" => Req::Step {
            frames: unsigned(step, "frames", 1).clamp(1, MAX_STEP_FRAMES as u64) as u32,
        },
        "shot" | "screenshot" => Req::Screenshot {
            view: step.get("view").and_then(|v| v.as_str()).map(|s| s.to_string()),
        },
        other => Req::Unknown(format!("batch op: {other}")),
    }
}
