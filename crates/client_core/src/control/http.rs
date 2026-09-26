//! Minimal HTTP transport for KTRL: a `TcpListener` thread with hand-rolled
//! request parsing, talking to the ECS through a channel.
//!
//! No async runtime — one thread per connection, each waiting (with a timeout)
//! for the ECS to answer.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::Duration;

use super::batch::{mouse_request, parse_batch_step};
use super::events;
use super::{BODY_LIMIT, MAX_STEP_FRAMES, RESPONSE_TIMEOUT};

// --- Wire types -------------------------------------------------------------

pub(super) struct Response {
    pub(super) status: u16,
    pub(super) content_type: &'static str,
    pub(super) body: Vec<u8>,
}

impl Response {
    pub(super) fn json(body: String) -> Self {
        Self { status: 200, content_type: "application/json", body: body.into_bytes() }
    }
    pub(super) fn text(status: u16, body: String) -> Self {
        Self { status, content_type: "text/plain; charset=utf-8", body: body.into_bytes() }
    }
    pub(super) fn png(body: Vec<u8>) -> Self {
        Self { status: 200, content_type: "image/png", body }
    }
}

pub(super) enum Req {
    State,
    Version,
    Screenshot { view: Option<String> },
    Pause { on: bool },
    Step { frames: u32 },
    Action { name: String, args: serde_json::Value },
    UiQuery { label: Option<String> },
    Logs { since: Option<u64>, tail: usize, level: Option<String> },
    SceneTree { root: Option<u32>, depth: usize },
    EntityDetail { id: u32 },
    MeshInfo { id: u32 },
    MaterialInfo { id: u32 },
    Key { key: String, action: String },
    Text { text: String },
    /// Same-frame sequence of requests (see `batch::parse_batch_step`).
    Batch { steps: Vec<Req> },
    MouseMove { x: f32, y: f32 },
    MouseButton { button: String, action: String },
    /// `action`: `click` (one frame), `hover` (held), `unhover`.
    UiClick { label: String, action: String },
    Unknown(String),
}

pub(super) struct Envelope {
    pub(super) req: Req,
    pub(super) resp: Sender<Response>,
}

// --- Server thread ----------------------------------------------------------

pub(super) fn start_server(
    port: u16,
    token: Option<Arc<str>>,
) -> std::io::Result<Receiver<Envelope>> {
    let (tx, rx) = mpsc::channel::<Envelope>();
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let tx = tx.clone();
            let token = token.clone();
            std::thread::spawn(move || {
                let _ = handle_connection(stream, tx, token);
            });
        }
    });
    Ok(rx)
}

fn authorized(expected: &str, header: Option<&str>) -> bool {
    header
        .and_then(|h| h.strip_prefix("Bearer ").or_else(|| h.strip_prefix("bearer ")))
        .map(|t| t.trim() == expected)
        .unwrap_or(false)
}

fn handle_connection(
    mut stream: TcpStream,
    tx: Sender<Envelope>,
    token: Option<Arc<str>>,
) -> std::io::Result<()> {
    let (method, path, body, auth) = read_request(&mut stream)?;
    if let Some(token) = &token {
        if !authorized(token, auth.as_deref()) {
            return write_response(&mut stream, &Response::text(401, "unauthorized".into()));
        }
    }
    // `/events` is a stream, not a request/response call: keep the connection open.
    if path.split('?').next() == Some("/events") {
        return stream_events(stream);
    }
    let req = parse_request(&method, &path, &body);

    let (resp_tx, resp_rx) = mpsc::channel();
    if tx.send(Envelope { req, resp: resp_tx }).is_err() {
        let _ = write_response(&mut stream, &Response::text(503, "app not running".into()));
        return Ok(());
    }

    let response = match resp_rx.recv_timeout(RESPONSE_TIMEOUT) {
        Ok(r) => r,
        Err(_) => Response::text(504, "timeout waiting for the frame".into()),
    };
    write_response(&mut stream, &response)
}

// --- Request parsing --------------------------------------------------------

fn read_request(stream: &mut TcpStream) -> std::io::Result<(String, String, Vec<u8>, Option<String>)> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];

    // Read until end of headers.
    let header_end;
    loop {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "eof"));
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
            header_end = pos + 4;
            break;
        }
        if buf.len() > BODY_LIMIT {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "headers too large"));
        }
    }

    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.lines();
    let request_line = lines.next().unwrap_or_default().to_string();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();

    let content_length: usize;
    let mut auth = None;
    let mut content_length_opt: Option<usize> = None;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            if k.eq_ignore_ascii_case("content-length") {
                content_length_opt = v.trim().parse().ok();
            } else if k.eq_ignore_ascii_case("authorization") {
                auth = Some(v.trim().to_string());
            }
        }
    }
    content_length = content_length_opt.unwrap_or(0);

    let mut body = buf[header_end..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(content_length.min(BODY_LIMIT));

    Ok((method, path, body, auth))
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(v.replace('+', " ").replace("%20", " "));
            }
        }
    }
    None
}

pub(super) fn parse_request(method: &str, path: &str, body: &[u8]) -> Req {
    let (path, query) = match path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (path, ""),
    };
    let json: serde_json::Value = serde_json::from_slice(body).unwrap_or(serde_json::Value::Null);
    let label_param = json
        .get("label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| query_param(query, "label"));

    // REST-ish short forms: `/entity/42`, `/mesh/42`, `/material/42`.
    if let Some(id) = path.strip_prefix("/entity/").and_then(|s| s.parse::<u32>().ok()) {
        return Req::EntityDetail { id };
    }
    if let Some(id) = path.strip_prefix("/mesh/").and_then(|s| s.parse::<u32>().ok()) {
        return Req::MeshInfo { id };
    }
    if let Some(id) = path.strip_prefix("/material/").and_then(|s| s.parse::<u32>().ok()) {
        return Req::MaterialInfo { id };
    }

    match (method, path) {
        ("GET", "/state") | ("GET", "/") => Req::State,
        ("GET", "/version") => Req::Version,
        ("GET", "/screenshot") | ("POST", "/screenshot") => Req::Screenshot {
            view: query_param(query, "view").or_else(|| {
                json.get("view").and_then(|v| v.as_str()).map(|s| s.to_string())
            }),
        },
        ("GET", "/scene_tree") => Req::SceneTree {
            root: query_param(query, "root").and_then(|s| s.parse().ok()),
            depth: query_param(query, "depth").and_then(|s| s.parse().ok()).unwrap_or(4),
        },
        ("GET", "/entity") => match query_param(query, "id").and_then(|s| s.parse().ok()) {
            Some(id) => Req::EntityDetail { id },
            None => Req::Unknown("GET /entity (missing id)".into()),
        },
        ("GET", "/mesh") => match query_param(query, "id").and_then(|s| s.parse().ok()) {
            Some(id) => Req::MeshInfo { id },
            None => Req::Unknown("GET /mesh (missing id)".into()),
        },
        ("GET", "/material") => match query_param(query, "id").and_then(|s| s.parse().ok()) {
            Some(id) => Req::MaterialInfo { id },
            None => Req::Unknown("GET /material (missing id)".into()),
        },
        ("GET", "/ui_query") | ("POST", "/ui_query") => Req::UiQuery { label: label_param },
        ("GET", "/logs") => Req::Logs {
            since: query_param(query, "since").and_then(|s| s.parse().ok()),
            tail: query_param(query, "tail").and_then(|s| s.parse().ok()).unwrap_or(100),
            level: query_param(query, "level"),
        },
        ("POST", "/pause") => Req::Pause {
            on: json.get("on").and_then(|v| v.as_bool()).unwrap_or(true),
        },
        ("POST", "/step") => Req::Step {
            frames: json
                .get("frames")
                .and_then(|v| v.as_u64())
                .unwrap_or(1)
                .clamp(1, MAX_STEP_FRAMES as u64) as u32,
        },
        ("POST", "/action") => Req::Action {
            name: json.get("action").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            args: json.clone(),
        },
        ("POST", "/batch") => Req::Batch {
            steps: json
                .get("steps")
                .and_then(|v| v.as_array())
                .map(|steps| steps.iter().map(parse_batch_step).collect())
                .unwrap_or_default(),
        },
        ("POST", "/key") => Req::Key {
            key: json.get("key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            action: json.get("action").and_then(|v| v.as_str()).unwrap_or("tap").to_string(),
        },
        ("POST", "/text") => Req::Text {
            text: json.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        },
        ("POST", "/ui_click") | ("POST", "/ui_hover") => Req::UiClick {
            label: label_param.unwrap_or_default(),
            action: json
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or(if path == "/ui_hover" { "hover" } else { "click" })
                .to_string(),
        },
        ("POST", "/mouse") => mouse_request(&json),
        _ => Req::Unknown(format!("{} {}", method, path)),
    }
}

// --- Response ---------------------------------------------------------------

fn reason_phrase(status: u16) -> &'static str {
    match status {
        401 => "Unauthorized",
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "OK",
    }
}

fn write_response(stream: &mut TcpStream, resp: &Response) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n",
        resp.status,
        reason_phrase(resp.status),
        resp.content_type,
        resp.body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&resp.body)?;
    stream.flush()
}

// --- Server-Sent Events -----------------------------------------------------

/// Stream `/events` to this client until it disconnects.
fn stream_events(mut stream: TcpStream) -> std::io::Result<()> {
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\
                Cache-Control: no-cache\r\nConnection: keep-alive\r\n\
                Access-Control-Allow-Origin: *\r\nX-Accel-Buffering: no\r\n\r\n";
    stream.write_all(head.as_bytes())?;
    stream.write_all(b": connected\n\n")?;
    stream.flush()?;

    let rx = events::subscribe();
    loop {
        match rx.recv_timeout(Duration::from_secs(15)) {
            Ok(event) => {
                let frame = format!("event: {}\ndata: {}\n\n", event.kind, event.data);
                if stream.write_all(frame.as_bytes()).is_err() || stream.flush().is_err() {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // Comment frame keeps proxies from closing an idle stream.
                if stream.write_all(b": keep-alive\n\n").is_err() || stream.flush().is_err() {
                    break;
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

/// Basename of the running executable, reported by `GET /version`.
pub(super) fn detect_binary() -> String {
    std::env::args()
        .next()
        .as_deref()
        .map(std::path::Path::new)
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".exe")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_routes() {
        assert!(matches!(parse_request("GET", "/state", &[]), Req::State));
        assert!(matches!(parse_request("GET", "/", &[]), Req::State));
        assert!(matches!(parse_request("GET", "/version", &[]), Req::Version));
        assert!(matches!(
            parse_request("GET", "/screenshot", &[]),
            Req::Screenshot { view: None }
        ));
        assert!(matches!(
            parse_request("GET", "/screenshot?view=camera:42", &[]),
            Req::Screenshot { view: Some(v) } if v == "camera:42"
        ));
        assert!(matches!(
            parse_request("POST", "/screenshot", br#"{"view":"image"}"#),
            Req::Screenshot { view: Some(v) } if v == "image"
        ));
        assert!(matches!(
            parse_request("GET", "/entity/42", &[]),
            Req::EntityDetail { id: 42 }
        ));
        assert!(matches!(
            parse_request("GET", "/mesh/7", &[]),
            Req::MeshInfo { id: 7 }
        ));
        assert!(matches!(
            parse_request("GET", "/entity?id=42", &[]),
            Req::EntityDetail { id: 42 }
        ));
        assert!(matches!(
            parse_request("GET", "/scene_tree?depth=2&root=5", &[]),
            Req::SceneTree { root: Some(5), depth: 2 }
        ));
    }

    #[test]
    fn step_frames_are_clamped() {
        assert!(matches!(
            parse_request("POST", "/step", br#"{"frames":5}"#),
            Req::Step { frames: 5 }
        ));
        assert!(matches!(
            parse_request("POST", "/step", &[]),
            Req::Step { frames: 1 }
        ));
        assert!(matches!(
            parse_request("POST", "/step", br#"{"frames":1000}"#),
            Req::Step { frames: MAX_STEP_FRAMES }
        ));
    }

    #[test]
    fn unknown_and_incomplete_routes_are_reported() {
        assert!(matches!(parse_request("GET", "/nope", &[]), Req::Unknown(_)));
        assert!(matches!(parse_request("GET", "/entity", &[]), Req::Unknown(_)));
        assert!(matches!(parse_request("GET", "/mesh", &[]), Req::Unknown(_)));
    }

    #[test]
    fn bearer_auth() {
        assert!(authorized("abc", Some("Bearer abc")));
        assert!(authorized("abc", Some("bearer abc")));
        assert!(!authorized("abc", Some("Bearer xyz")));
        assert!(!authorized("abc", Some("abc")));
        assert!(!authorized("abc", None));
    }

    #[test]
    fn parses_batch_steps() {
        let body = br#"{"steps":[
            {"op":"action","name":"Undo"},
            {"op":"key","key":"Enter"},
            {"op":"pause","on":false},
            {"op":"ui_click","label":"Cube"},
            {"op":"state"}
        ]}"#;
        let Req::Batch { steps } = parse_request("POST", "/batch", body) else {
            panic!("expected a batch");
        };
        assert_eq!(steps.len(), 5);
        assert!(matches!(steps[0], Req::Action { ref name, .. } if name == "Undo"));
        assert!(matches!(steps[1], Req::Key { ref key, .. } if key == "Enter"));
        assert!(matches!(steps[2], Req::Pause { on: false }));
        assert!(matches!(steps[3], Req::UiClick { ref label, .. } if label == "Cube"));
        assert!(matches!(steps[4], Req::State));
    }
}
