//! Conditional watchpoints: pause and report when a predicate holds.
//!
//! A watch is evaluated every frame (state predicates against a snapshot of
//! `GET /state`, entity predicates against the ECS). On a hit the app is softly
//! paused (virtual time, so rendering and the HTTP server keep running), a
//! `watch` event with a state snapshot is published on `/events`, and an
//! optional screenshot is written to disk.
//!
//! ```jsonc
//! // state predicate: fire when map cell (3,4) is higher than 2
//! {"field":"map.cells.3.4.h","op":">","value":2}
//! // entity predicate: fire when a mesh named "probe" is within 1.0 of (5,1,5)
//! {"name":"probe","component":"mesh","near":[5,1,5],"radius":1.0}
//! // options: "pause" (default true), "snapshot" (true), "screenshot" (false),
//! //          "once" (true), "dir" ("/tmp")
//! ```

use bevy::prelude::*;
use serde_json::{json, Value};

/// Maximum number of watches kept at once.
pub(super) const MAX_WATCHES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Contains,
    Changed,
}

impl CmpOp {
    fn parse(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "==" | "=" | "eq" => CmpOp::Eq,
            "!=" | "ne" => CmpOp::Ne,
            "<" | "lt" => CmpOp::Lt,
            "<=" | "le" => CmpOp::Le,
            ">" | "gt" => CmpOp::Gt,
            ">=" | "ge" => CmpOp::Ge,
            "contains" => CmpOp::Contains,
            "changed" => CmpOp::Changed,
            _ => return None,
        })
    }

    fn label(&self) -> &'static str {
        match self {
            CmpOp::Eq => "==",
            CmpOp::Ne => "!=",
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
            CmpOp::Contains => "contains",
            CmpOp::Changed => "changed",
        }
    }
}

/// Entity selector used by entity predicates.
#[derive(Debug, Clone, Default)]
pub(super) struct EntityFilter {
    /// Case-insensitive substring of the entity's `Name`.
    pub name: Option<String>,
    /// One of `mesh`, `camera`, `ui_node`, `ui_text`, `light`, `transform`, `any`.
    pub component: Option<String>,
    /// World-space point to stay within `radius` of.
    pub near: Option<Vec3>,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub(super) enum Predicate {
    State {
        field: String,
        op: CmpOp,
        value: Value,
        previous: Option<Value>,
    },
    Entity {
        filter: EntityFilter,
        op: CmpOp,
        count: f64,
        previous: Option<f64>,
    },
}

#[derive(Debug, Clone)]
pub(super) struct Watch {
    pub id: u64,
    pub predicate: Predicate,
    pub pause: bool,
    pub snapshot: bool,
    pub screenshot: bool,
    pub once: bool,
    pub dir: String,
    pub fired: bool,
    /// Edge trigger for repeating watches: re-armed when the predicate is false.
    pub armed: bool,
    pub hits: u64,
    pub last_frame: Option<u64>,
    pub last: Option<Value>,
}

/// A predicate that just became true.
pub(super) struct Hit {
    pub id: u64,
    pub frame: u64,
    pub pause: bool,
    pub snapshot: Option<Value>,
    pub screenshot: bool,
    pub dir: String,
}

impl Watch {
    /// Parse `POST /watch`'s body. `id` is assigned by the caller.
    pub(super) fn parse(spec: &Value) -> Result<Self, String> {
        let predicate = if let Some(field) = spec.get("field").and_then(|v| v.as_str()) {
            let op = op_from(spec, "op", CmpOp::Eq)?;
            let value = spec.get("value").cloned().unwrap_or(Value::Null);
            if op != CmpOp::Changed && value.is_null() {
                return Err("state watch needs a `value` (or op `changed`)".into());
            }
            Predicate::State { field: field.to_string(), op, value, previous: None }
        } else if spec.get("name").is_some()
            || spec.get("component").is_some()
            || spec.get("near").is_some()
        {
            let filter = EntityFilter {
                name: spec.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                component: spec.get("component").and_then(|v| v.as_str()).map(|s| s.to_string()),
                near: spec.get("near").and_then(vec3),
                radius: spec.get("radius").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
            };
            Predicate::Entity {
                filter,
                op: op_from(spec, "count_op", CmpOp::Ge)?,
                count: spec.get("count").and_then(|v| v.as_f64()).unwrap_or(1.0),
                previous: None,
            }
        } else {
            return Err("watch needs `field` or `name`/`component`/`near`".into());
        };

        let dir = spec
            .get("dir")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());

        Ok(Self {
            id: 0,
            predicate,
            pause: spec.get("pause").and_then(|v| v.as_bool()).unwrap_or(true),
            snapshot: spec.get("snapshot").and_then(|v| v.as_bool()).unwrap_or(true),
            screenshot: spec.get("screenshot").and_then(|v| v.as_bool()).unwrap_or(false),
            once: spec.get("once").and_then(|v| v.as_bool()).unwrap_or(true),
            dir,
            fired: false,
            armed: true,
            hits: 0,
            last_frame: None,
            last: None,
        })
    }

    pub(super) fn is_state_predicate(&self) -> bool {
        matches!(self.predicate, Predicate::State { .. })
    }

    pub(super) fn active(&self) -> bool {
        !(self.once && self.fired)
    }

    /// Serializable description, optionally with the last snapshot.
    pub(super) fn to_json(&self, include_last: bool) -> Value {
        let mut obj = json!({
            "id": self.id,
            "fired": self.fired,
            "hits": self.hits,
            "last_frame": self.last_frame,
            "pause": self.pause,
            "once": self.once,
        });
        match &self.predicate {
            Predicate::State { field, op, value, .. } => {
                obj["kind"] = json!("state");
                obj["field"] = json!(field);
                obj["op"] = json!(op.label());
                obj["value"] = value.clone();
            }
            Predicate::Entity { filter, op, count, .. } => {
                obj["kind"] = json!("entity");
                obj["name"] = json!(filter.name);
                obj["component"] = json!(filter.component);
                obj["near"] = filter
                    .near
                    .map(|p| json!([p.x, p.y, p.z]))
                    .unwrap_or(Value::Null);
                obj["radius"] = json!(filter.radius);
                obj["count_op"] = json!(op.label());
                obj["count"] = json!(count);
            }
        }
        if include_last {
            obj["last"] = self.last.clone().unwrap_or(Value::Null);
        }
        obj
    }
}

/// Evaluate all active watches, updating their internal state.
///
/// `count_entities` supplies the entity count for entity predicates (passed as
/// a closure so this module stays testable without an ECS).
pub(super) fn evaluate(
    watches: &mut [Watch],
    frame: u64,
    state: Option<&Value>,
    count_entities: &dyn Fn(&EntityFilter) -> usize,
) -> Vec<Hit> {
    let mut hits = Vec::new();
    for watch in watches.iter_mut() {
        if !watch.active() {
            continue;
        }
        let matched = match &mut watch.predicate {
            Predicate::State { field, op, value, previous } => {
                let Some(state) = state else { continue };
                let current = lookup(state, field);
                let matched = if *op == CmpOp::Changed {
                    previous.as_ref().is_some_and(|prev| current != Some(prev))
                } else {
                    compare(current, op, value)
                };
                *previous = current.cloned();
                matched
            }
            Predicate::Entity { filter, op, count, previous } => {
                let current = count_entities(filter) as f64;
                let matched = if *op == CmpOp::Changed {
                    previous.is_some_and(|prev| (prev - current).abs() > f64::EPSILON)
                } else {
                    compare_count(current, op, *count)
                };
                *previous = Some(current);
                matched
            }
        };

        if matched && watch.armed {
            watch.armed = false;
            watch.fired = true;
            watch.hits += 1;
            watch.last_frame = Some(frame);
            let snapshot = if watch.snapshot { state.cloned() } else { None };
            watch.last = snapshot.clone();
            hits.push(Hit {
                id: watch.id,
                frame,
                pause: watch.pause,
                snapshot,
                screenshot: watch.screenshot,
                dir: watch.dir.clone(),
            });
        } else if !matched {
            watch.armed = true;
        }
    }
    hits
}

fn op_from(spec: &Value, key: &str, default: CmpOp) -> Result<CmpOp, String> {
    match spec.get(key).and_then(|v| v.as_str()) {
        Some(raw) => CmpOp::parse(raw).ok_or_else(|| format!("unknown op: {raw}")),
        None => Ok(default),
    }
}

fn vec3(value: &Value) -> Option<Vec3> {
    let array = value.as_array()?;
    if array.len() < 3 {
        return None;
    }
    Some(Vec3::new(
        array[0].as_f64()? as f32,
        array[1].as_f64()? as f32,
        array[2].as_f64()? as f32,
    ))
}

/// Resolve a dot path such as `map.cells.3.4.h` inside a JSON value.
pub(super) fn lookup<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(part)?,
            Value::Array(items) => items.get(part.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

fn compare(current: Option<&Value>, op: &CmpOp, expected: &Value) -> bool {
    let Some(current) = current else { return false };
    match op {
        CmpOp::Eq => current == expected,
        CmpOp::Ne => current != expected,
        CmpOp::Contains => match current {
            Value::String(text) => expected.as_str().map(|e| text.contains(e)).unwrap_or(false),
            Value::Array(items) => items.iter().any(|item| item == expected),
            Value::Object(map) => expected.as_str().map(|k| map.contains_key(k)).unwrap_or(false),
            _ => false,
        },
        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge => {
            if let (Some(a), Some(b)) = (current.as_f64(), expected.as_f64()) {
                return compare_count(a, op, b);
            }
            match (current.as_str(), expected.as_str()) {
                (Some(a), Some(b)) => match op {
                    CmpOp::Lt => a < b,
                    CmpOp::Le => a <= b,
                    CmpOp::Gt => a > b,
                    CmpOp::Ge => a >= b,
                    _ => false,
                },
                _ => false,
            }
        }
        CmpOp::Changed => false,
    }
}

fn compare_count(current: f64, op: &CmpOp, expected: f64) -> bool {
    match op {
        CmpOp::Lt => current < expected,
        CmpOp::Le => current <= expected,
        CmpOp::Gt => current > expected,
        CmpOp::Ge => current >= expected,
        CmpOp::Eq => (current - expected).abs() <= f64::EPSILON,
        CmpOp::Ne => (current - expected).abs() > f64::EPSILON,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_state_watch() {
        let watch = Watch::parse(&json!({"field":"map.cells.3.4.h","op":">","value":2})).unwrap();
        assert!(watch.is_state_predicate());
        assert!(watch.pause && watch.once);
        match watch.predicate {
            Predicate::State { ref field, ref op, .. } => {
                assert_eq!(field, "map.cells.3.4.h");
                assert_eq!(*op, CmpOp::Gt);
            }
            _ => panic!("expected state predicate"),
        }
    }

    #[test]
    fn parses_entity_watch() {
        let watch =
            Watch::parse(&json!({"name":"probe","component":"mesh","near":[1,2,3],"radius":0.5}))
                .unwrap();
        assert!(!watch.is_state_predicate());
        match watch.predicate {
            Predicate::Entity { filter, op, count, .. } => {
                assert_eq!(filter.name.as_deref(), Some("probe"));
                assert_eq!(filter.near, Some(Vec3::new(1.0, 2.0, 3.0)));
                assert_eq!(op, CmpOp::Ge);
                assert_eq!(count, 1.0);
            }
            _ => panic!("expected entity predicate"),
        }
    }

    #[test]
    fn rejects_empty_and_unknown() {
        assert!(Watch::parse(&json!({})).is_err());
        assert!(Watch::parse(&json!({"field":"frame","op":"~","value":1})).is_err());
        assert!(Watch::parse(&json!({"field":"frame"})).is_err());
    }

    #[test]
    fn compares_scalars_and_strings() {
        let state = json!({"frame": 12, "game_state": "InGame", "map": {"cells": [{"h": 3}]}});
        assert!(compare(lookup(&state, "frame"), &CmpOp::Gt, &json!(10)));
        assert!(compare(lookup(&state, "game_state"), &CmpOp::Contains, &json!("Game")));
        assert!(compare(lookup(&state, "map.cells.0.h"), &CmpOp::Eq, &json!(3)));
        assert!(!compare(lookup(&state, "map.cells.9.h"), &CmpOp::Eq, &json!(3)));
    }

    #[test]
    fn edge_triggered_repeat_fires_on_rising_edge_only() {
        let mut watches = vec![{
            let mut w = Watch::parse(&json!({"field":"v","op":">","value":1,"once":false})).unwrap();
            w.id = 7;
            w
        }];
        let no_entities = |_: &EntityFilter| 0usize;

        // v = 0 -> no hit, armed.
        let hit = evaluate(&mut watches, 1, Some(&json!({"v": 0})), &no_entities);
        assert!(hit.is_empty() && watches[0].armed);

        // v = 5 -> rising edge fires once.
        let hits = evaluate(&mut watches, 2, Some(&json!({"v": 5})), &no_entities);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].frame, 2);
        assert!(watches[0].fired && !watches[0].armed);

        // still true -> no repeat.
        let hits = evaluate(&mut watches, 3, Some(&json!({"v": 5})), &no_entities);
        assert!(hits.is_empty());

        // false then true again -> fires again.
        let hits = evaluate(&mut watches, 4, Some(&json!({"v": 0})), &no_entities);
        assert!(hits.is_empty() && watches[0].armed);
        let hits = evaluate(&mut watches, 5, Some(&json!({"v": 9})), &no_entities);
        assert_eq!(hits.len(), 1);
        assert_eq!(watches[0].hits, 2);
    }

    #[test]
    fn entity_predicate_uses_the_count_closure() {
        let mut watches = vec![{
            let mut w = Watch::parse(&json!({"name":"probe","component":"mesh","count":2})).unwrap();
            w.id = 1;
            w
        }];
        let count = |_: &EntityFilter| 2usize;
        let hits = evaluate(&mut watches, 10, None, &count);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].snapshot.is_none()); // no state snapshot requested
    }
}
