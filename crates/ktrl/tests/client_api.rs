//! Integration test for `ktrl::Client` against a stub HTTP server.
//!
//! Verifies that each typed method builds the right request (method, path,
//! body, auth header) and parses the response — without needing a running
//! Klep2tron instance.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver};
use std::thread;

/// One request as seen by the stub.
#[derive(Debug)]
struct Recorded {
    method: String,
    path: String,
    auth: Option<String>,
    body: String,
}

struct Stub {
    base: String,
    requests: Receiver<Recorded>,
}

impl Stub {
    /// Serve `responses` (one per connection) in order, recording requests.
    fn start(responses: Vec<(u16, &'static str, &'static str)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (tx, requests) = mpsc::channel();

        thread::spawn(move || {
            for (status, content_type, body) in responses {
                let Ok((mut stream, _)) = listener.accept() else { break };
                let mut reader = BufReader::new(stream.try_clone().unwrap());

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                let mut parts = request_line.split_whitespace();
                let method = parts.next().unwrap_or_default().to_string();
                let path = parts.next().unwrap_or_default().to_string();

                let mut auth = None;
                let mut content_length = 0usize;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                        break;
                    }
                    if let Some((key, value)) = line.split_once(':') {
                        if key.eq_ignore_ascii_case("authorization") {
                            auth = Some(value.trim().to_string());
                        } else if key.eq_ignore_ascii_case("content-length") {
                            content_length = value.trim().parse().unwrap_or(0);
                        }
                    }
                }
                let mut body_bytes = vec![0u8; content_length];
                if content_length > 0 {
                    reader.read_exact(&mut body_bytes).unwrap();
                }

                tx.send(Recorded {
                    method,
                    path,
                    auth,
                    body: String::from_utf8_lossy(&body_bytes).into_owned(),
                })
                .ok();

                let reason = if status == 200 { "OK" } else { "Error" };
                let head = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(head.as_bytes()).unwrap();
                stream.write_all(body.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        });

        Self { base, requests }
    }

    fn next(&self) -> Recorded {
        self.requests.recv().unwrap()
    }
}

#[test]
fn sends_typed_requests_and_parses_responses() {
    let stub = Stub::start(vec![
        (200, "application/json", r#"{"name":"KTRL","api":1}"#),
        (200, "application/json", r#"{"game_state":"InGame"}"#),
        (200, "application/json", r#"{"ok":true,"frame":12}"#),
        (200, "application/json", r#"{"ok":true,"action":"Undo"}"#),
        (200, "application/json", r#"{"ok":true,"action":"SetTile"}"#),
    ]);
    let client = ktrl::Client::new(&stub.base, Some("s3cret".into()));

    assert_eq!(client.version().unwrap()["api"], 1);
    assert_eq!(client.state().unwrap()["game_state"], "InGame");
    assert!(client.step(7).unwrap().contains("\"frame\":12"));
    client.action("Undo", None).unwrap();
    client.set_tile(2, 3, Some(1), Some("WedgeN")).unwrap();

    let version = stub.next();
    assert_eq!((version.method.as_str(), version.path.as_str()), ("GET", "/version"));
    assert_eq!(version.auth.as_deref(), Some("Bearer s3cret"));

    let state = stub.next();
    assert_eq!(state.path, "/state");

    let step = stub.next();
    assert_eq!((step.method.as_str(), step.path.as_str()), ("POST", "/step"));
    assert_eq!(step.body, r#"{"frames":7}"#);

    let action = stub.next();
    assert_eq!(action.path, "/action");
    assert!(action.body.contains("\"action\":\"Undo\""));

    let tile = stub.next();
    let tile_body: serde_json::Value = serde_json::from_str(&tile.body).unwrap();
    assert_eq!(tile_body["action"], "SetTile");
    assert_eq!(tile_body["x"], 2);
    assert_eq!(tile_body["tt"], "WedgeN");
}

#[test]
fn mutations_and_batch_build_expected_bodies() {
    let stub = Stub::start(vec![
        (200, "application/json", r#"{"ok":true,"action":"SpawnEntity"}"#),
        (200, "application/json", r#"{"ok":true,"action":"DespawnEntity"}"#),
        (200, "application/json", r#"{"ok":true,"action":"SetTransform"}"#),
        (200, "application/json", r#"{"results":[]}"#),
        (200, "application/json", r#"{"ok":true,"chars":3}"#),
    ]);
    let client = ktrl::Client::new(&stub.base, None);

    client.spawn_entity(serde_json::json!({"mesh":"cube","name":"p"})).unwrap();
    client.despawn_entity(42).unwrap();
    client.set_transform(42, serde_json::json!({"translation":[1,2,3],"relative":true})).unwrap();
    client.batch(serde_json::json!([{"op":"state"}])).unwrap();
    client.text("abc").unwrap();

    let spawn: serde_json::Value = serde_json::from_str(&stub.next().body).unwrap();
    assert_eq!(spawn["action"], "SpawnEntity");
    assert_eq!(spawn["mesh"], "cube");

    let despawn: serde_json::Value = serde_json::from_str(&stub.next().body).unwrap();
    assert_eq!(despawn["id"], 42);

    let transform: serde_json::Value = serde_json::from_str(&stub.next().body).unwrap();
    assert_eq!(transform["action"], "SetTransform");
    assert_eq!(transform["id"], 42);
    assert_eq!(transform["relative"], true);

    let batch: serde_json::Value = serde_json::from_str(&stub.next().body).unwrap();
    assert_eq!(batch["steps"][0]["op"], "state");

    let text: serde_json::Value = serde_json::from_str(&stub.next().body).unwrap();
    assert_eq!(text["text"], "abc");
}

#[test]
fn maps_http_errors_and_returns_raw_png() {
    let stub = Stub::start(vec![
        (401, "text/plain", "unauthorized"),
        (200, "image/png", "PNGDATA"),
    ]);
    let client = ktrl::Client::new(&stub.base, Some("wrong".into()));

    match client.version() {
        Err(ktrl::Error::Status { code, body }) => {
            assert_eq!(code, 401);
            assert_eq!(body, "unauthorized");
        }
        other => panic!("expected a 401 status error, got {other:?}"),
    }

    let png = client.screenshot(None).unwrap();
    assert_eq!(png, b"PNGDATA");
}
