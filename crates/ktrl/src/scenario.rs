//! Declarative scenario runner and frame recorder for `ktrl`.
//!
//! A scenario is a JSON list of deterministic steps (`frames` blocks until the
//! requested frames rendered, so no wall-clock sleeps are needed):
//!
//! ```json
//! {
//!   "name": "editor previews",
//!   "steps": [
//!     {"op": "action", "name": "StartEditor"},
//!     {"op": "frames", "frames": 60},
//!     {"op": "shot", "path": "/tmp/a.png"},
//!     {"op": "ui_click", "label": "Wedge S"},
//!     {"op": "frames", "frames": 30},
//!     {"op": "wait_state", "field": "game_state", "equals": "InGame"},
//!     {"op": "shot", "path": "/tmp/b.png", "view": "camera:529"},
//!     {"op": "expect_state", "field": "map.size.0", "equals": 16}
//!   ]
//! }
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::{Client, Error, Result};

#[derive(Debug, Deserialize)]
pub struct Scenario {
    #[serde(default)]
    pub name: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Step {
    /// Run a KTRL action; `set` merges extra fields into the body.
    Action {
        name: String,
        #[serde(default)]
        set: Option<serde_json::Value>,
    },
    /// Advance exactly this many frames (blocks until rendered).
    Frames { frames: u32 },
    Pause { on: bool },
    /// Capture a PNG to `path` (`view`: `primary`/`camera:<id>`/`rtt:<id>`).
    Shot {
        path: String,
        #[serde(default)]
        view: Option<String>,
    },
    UiClick { label: String },
    UiHover { label: String },
    Unhover,
    Key {
        key: String,
        #[serde(default = "default_tap")]
        action: String,
    },
    SetTile {
        x: u32,
        z: u32,
        #[serde(default)]
        h: Option<i32>,
        #[serde(default)]
        tt: Option<String>,
    },
    /// Poll `/state` until `field` (dot path) equals `equals`.
    WaitState {
        field: String,
        equals: serde_json::Value,
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    },
    /// Assert `field` equals `equals` right now.
    ExpectState {
        field: String,
        equals: serde_json::Value,
    },
}

fn default_tap() -> String {
    "tap".to_string()
}

fn default_timeout() -> u64 {
    5000
}

/// Resolve a dot path such as `map.size.0` inside a JSON value.
pub fn lookup<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for part in path.split('.') {
        current = match current {
            serde_json::Value::Object(map) => map.get(part)?,
            serde_json::Value::Array(items) => items.get(part.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

impl Scenario {
    pub fn from_json(text: &str) -> Result<Self> {
        Ok(serde_json::from_str(text)?)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_json(&std::fs::read_to_string(path)?)
    }
}

/// Run every step in order. Returns the number of steps executed.
pub fn run(client: &Client, scenario: &Scenario, verbose: bool) -> Result<usize> {
    for (index, step) in scenario.steps.iter().enumerate() {
        if verbose {
            eprintln!("[{index}] {step:?}");
        }
        match step {
            Step::Action { name, set } => {
                client.action(name, set.clone())?;
            }
            Step::Frames { frames } => {
                client.step(*frames)?;
            }
            Step::Pause { on } => {
                client.pause(*on)?;
            }
            Step::Shot { path, view } => {
                let bytes = client.screenshot(view.as_deref())?;
                let path = PathBuf::from(path);
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                std::fs::write(&path, &bytes)?;
                if verbose {
                    eprintln!("    wrote {} ({} bytes)", path.display(), bytes.len());
                }
            }
            Step::UiClick { label } => {
                client.ui_click(label, "click")?;
            }
            Step::UiHover { label } => {
                client.ui_click(label, "hover")?;
            }
            Step::Unhover => {
                client.ui_click("", "unhover")?;
            }
            Step::Key { key, action } => {
                client.key(key, action)?;
            }
            Step::SetTile { x, z, h, tt } => {
                client.set_tile(*x, *z, *h, tt.as_deref())?;
            }
            Step::WaitState {
                field,
                equals,
                timeout_ms,
            } => {
                let deadline = Instant::now() + Duration::from_millis(*timeout_ms);
                loop {
                    let state = client.state()?;
                    match lookup(&state, field) {
                        Some(actual) if actual == equals => break,
                        _ if Instant::now() >= deadline => {
                            let actual = lookup(&state, field).cloned();
                            return Err(Error::Scenario(format!(
                                "timeout after {timeout_ms}ms waiting for {field} == {equals} (got {actual:?})"
                            )));
                        }
                        _ => std::thread::sleep(Duration::from_millis(50)),
                    }
                }
            }
            Step::ExpectState { field, equals } => {
                let state = client.state()?;
                let actual = lookup(&state, field);
                if actual != Some(equals) {
                    return Err(Error::Scenario(format!(
                        "{field} == {equals} expected, got {actual:?}"
                    )));
                }
            }
        }
    }
    Ok(scenario.steps.len())
}

/// Capture `frames` screenshots, one every `every` frames, into `out_dir`.
///
/// Returns the written paths. Use `--every 1` for a video-like sequence.
pub fn record(
    client: &Client,
    out_dir: impl AsRef<Path>,
    prefix: &str,
    frames: u32,
    every: u32,
    view: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let out_dir = out_dir.as_ref();
    std::fs::create_dir_all(out_dir)?;
    let stride = every.max(1);
    let mut written = Vec::new();

    for frame in 0..frames {
        if frame % stride == 0 {
            let bytes = client.screenshot(view)?;
            let path = out_dir.join(format!("{prefix}_{frame:04}.png"));
            std::fs::write(&path, &bytes)?;
            written.push(path);
        }
        if frame + 1 < frames {
            client.step(1)?;
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_step_kinds() {
        let scenario = Scenario::from_json(
            r#"{
                "name": "t",
                "steps": [
                    {"op": "action", "name": "StartEditor"},
                    {"op": "action", "name": "SetTile", "set": {"x": 1}},
                    {"op": "frames", "frames": 10},
                    {"op": "pause", "on": true},
                    {"op": "shot", "path": "/tmp/a.png"},
                    {"op": "shot", "path": "/tmp/b.png", "view": "camera:7"},
                    {"op": "ui_click", "label": "Cube"},
                    {"op": "ui_hover", "label": "Cube"},
                    {"op": "unhover"},
                    {"op": "key", "key": "Enter"},
                    {"op": "key", "key": "A", "action": "press"},
                    {"op": "set_tile", "x": 3, "z": 4, "h": 2, "tt": "WedgeN"},
                    {"op": "wait_state", "field": "game_state", "equals": "InGame"},
                    {"op": "expect_state", "field": "map.size.0", "equals": 16}
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(scenario.steps.len(), 14);
    }

    #[test]
    fn rejects_unknown_op() {
        assert!(Scenario::from_json(r#"{"steps":[{"op":"nope"}]}"#).is_err());
    }

    #[test]
    fn resolves_dot_paths() {
        let value: serde_json::Value = serde_json::json!({
            "game_state": "InGame",
            "map": {"size": [16, 16], "cells": [{"h": 3}]}
        });
        assert_eq!(lookup(&value, "game_state").unwrap(), "InGame");
        assert_eq!(lookup(&value, "map.size.1").unwrap(), 16);
        assert_eq!(lookup(&value, "map.cells.0.h").unwrap(), 3);
        assert!(lookup(&value, "map.nope").is_none());
    }
}
