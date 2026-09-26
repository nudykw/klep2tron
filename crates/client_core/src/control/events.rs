//! Server-Sent Events fan-out for `GET /events`.
//!
//! Log entries (`log`), gameplay/editor state changes (`state`) and panics
//! (`panic`) are broadcast to every connected SSE client. The registry holds a
//! `Sender` per connection; the sender is dropped automatically once the
//! client disconnects and the writer thread sees a send failure.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

pub struct Event {
    pub(super) kind: &'static str,
    pub(super) data: String,
}

static SUBSCRIBERS: OnceLock<Mutex<Vec<Sender<Event>>>> = OnceLock::new();

fn subscribers() -> &'static Mutex<Vec<Sender<Event>>> {
    SUBSCRIBERS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Register a new SSE client. Drop the receiver to unsubscribe.
pub fn subscribe() -> Receiver<Event> {
    let (tx, rx) = channel();
    subscribers().lock().unwrap().push(tx);
    rx
}

/// Send an event to every connected client. Dead subscriptions are pruned.
pub fn publish(kind: &'static str, data: String) {
    let mut subs = subscribers().lock().unwrap();
    if subs.is_empty() {
        return;
    }
    subs.retain(|tx| tx.send(Event { kind, data: data.clone() }).is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Drain until an event of the expected kind shows up (other tests may
    /// publish concurrently — the registry is global).
    fn recv_kind(rx: &Receiver<Event>, kind: &str) -> Event {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) if event.kind == kind => return event,
                Ok(_) => {}
                Err(_) if std::time::Instant::now() > deadline => panic!("no {kind} event"),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn publish_reaches_all_subscribers() {
        let a = subscribe();
        let b = subscribe();
        publish("test_event", r#"{"n":1}"#.into());
        assert_eq!(recv_kind(&a, "test_event").data, r#"{"n":1}"#);
        assert_eq!(recv_kind(&b, "test_event").data, r#"{"n":1}"#);
    }

    #[test]
    fn dead_subscribers_are_pruned() {
        let rx = subscribe();
        drop(rx);
        // Must not panic even though one receiver is gone.
        publish("test_event", "{}".into());
    }
}
