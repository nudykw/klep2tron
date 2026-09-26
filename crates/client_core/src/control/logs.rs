//! Bounded in-memory buffer of recent log lines, exposed via `GET /logs`.
//!
//! A custom [`tracing`] layer (installed through `LogPlugin::custom_layer`)
//! appends every event to a global ring buffer, so an agent can read warnings,
//! errors and panics without scraping stdout. Log order is globally monotonic
//! via `seq`, which also supports incremental polling (`?since=<seq>`).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, Once, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::app::App;
use bevy::log::tracing::field::{Field, Visit};
use bevy::log::tracing::{Event, Subscriber};
use bevy::log::tracing_subscriber::layer::{Context, Layer};

use super::events;

const CAPACITY: usize = 1000;

#[derive(Clone, serde::Serialize)]
pub struct LogEntry {
    pub seq: u64,
    pub ts_ms: u128,
    pub level: &'static str,
    pub target: String,
    pub message: String,
}

static BUFFER: OnceLock<Mutex<VecDeque<LogEntry>>> = OnceLock::new();
static SEQ: AtomicU64 = AtomicU64::new(1);

fn buffer() -> &'static Mutex<VecDeque<LogEntry>> {
    BUFFER.get_or_init(|| Mutex::new(VecDeque::with_capacity(CAPACITY)))
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn level_rank(level: &str) -> u8 {
    match level {
        "ERROR" => 4,
        "WARN" => 3,
        "INFO" => 2,
        "DEBUG" => 1,
        _ => 0,
    }
}

fn push(level: &'static str, target: &str, message: String) {
    let mut buf = buffer().lock().unwrap();
    if buf.len() >= CAPACITY {
        buf.pop_front();
    }
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let entry = LogEntry {
        seq,
        ts_ms: now_ms(),
        level,
        target: target.to_string(),
        message,
    };
    if let Ok(data) = serde_json::to_string(&entry) {
        events::publish("log", data);
    }
    buf.push_back(entry);
}

/// Forward panics to `/events` and the log buffer (best effort: the process may
/// exit before a slow client reads it).
pub fn install_panic_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let message = info.to_string();
            push("ERROR", "panic", message.clone());
            events::publish("panic", serde_json::json!({ "message": message }).to_string());
            previous(info);
        }));
    });
}

/// Build the `GET /logs` response.
pub(super) fn respond(since: Option<u64>, tail: usize, min_level: Option<&str>) -> super::http::Response {
    let (entries, last_seq) = snapshot(since, tail, min_level);
    let body = serde_json::json!({ "entries": entries, "last_seq": last_seq });
    super::http::Response::json(body.to_string())
}

/// Newest entries.
///
/// With `since` set, returns everything newer than that sequence (ignoring
/// `tail`); otherwise returns at most the last `tail` entries. `min_level`
/// filters by at least that severity (`info`, `warn`, `error`, …).
pub fn snapshot(since: Option<u64>, tail: usize, min_level: Option<&str>) -> (Vec<LogEntry>, u64) {
    let buf = buffer().lock().unwrap();
    let last_seq = buf.back().map(|e| e.seq).unwrap_or(0);
    let min_rank = min_level.map(|l| level_rank(&l.to_ascii_uppercase())).unwrap_or(0);

    let mut entries: Vec<LogEntry> = buf
        .iter()
        .filter(|e| since.map_or(true, |s| e.seq > s))
        .filter(|e| level_rank(e.level) >= min_rank)
        .cloned()
        .collect();

    if since.is_none() {
        let keep = tail.min(entries.len());
        entries.drain(..entries.len() - keep);
    }
    (entries, last_seq)
}

/// Install this as `LogPlugin::custom_layer` to capture logs into the buffer.
pub fn log_layer(_app: &mut App) -> Option<bevy::log::BoxedLayer> {
    install_panic_hook();
    Some(Box::new(LogCaptureLayer))
}

struct LogCaptureLayer;

#[derive(Default)]
struct FieldVisitor {
    message: String,
}

impl FieldVisitor {
    fn into_message(mut self) -> String {
        if self.message.is_empty() {
            self.message.push_str("(no message field)");
        }
        self.message
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            self.message.push_str(&format!("{}={value:?}", field.name()));
        }
    }
}

impl<S: Subscriber> Layer<S> for LogCaptureLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        let level = match *metadata.level() {
            bevy::log::tracing::Level::ERROR => "ERROR",
            bevy::log::tracing::Level::WARN => "WARN",
            bevy::log::tracing::Level::INFO => "INFO",
            bevy::log::tracing::Level::DEBUG => "DEBUG",
            bevy::log::tracing::Level::TRACE => "TRACE",
        };
        push(level, metadata.target(), visitor.into_message());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_filter_ranks() {
        assert!(level_rank("ERROR") > level_rank("WARN"));
        assert!(level_rank("WARN") > level_rank("INFO"));
        assert!(level_rank("INFO") > level_rank("DEBUG"));
        assert_eq!(level_rank("BOGUS"), 0);
    }

    #[test]
    fn snapshot_supports_incremental_reads() {
        for i in 0..5 {
            push("INFO", "test", format!("line {i}"));
        }
        let (all, last) = snapshot(None, 100, None);
        assert!(all.len() >= 5);
        assert_eq!(last, all.last().unwrap().seq);

        let (newer, _) = snapshot(Some(last), 0, None);
        assert!(newer.is_empty());

        push("WARN", "test", "warned".into());
        let (newer, _) = snapshot(Some(last), 0, None);
        assert_eq!(newer.len(), 1);
        assert_eq!(newer[0].level, "WARN");

        let (warns, _) = snapshot(None, 100, Some("warn"));
        assert!(warns.iter().all(|e| e.level == "WARN" || e.level == "ERROR"));
    }
}
