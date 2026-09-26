//! Minimal HTTP transport for KTRL: a `TcpListener` thread with hand-rolled
//! request parsing, talking to the ECS through a channel.
//!
//! No async runtime — one thread per connection, each waiting (with a timeout)
//! for the ECS to answer.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

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
    Screenshot,
    Pause { on: bool },
    Step { frames: u32 },
    Action { name: String, args: serde_json::Value },
    UiQuery { label: Option<String> },
    Key { key: String, action: String },
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

pub(super) fn start_server(port: u16) -> std::io::Result<Receiver<Envelope>> {
    let (tx, rx) = mpsc::channel::<Envelope>();
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let tx = tx.clone();
            std::thread::spawn(move || {
                let _ = handle_connection(stream, tx);
            });
        }
    });
    Ok(rx)
}

fn handle_connection(mut stream: TcpStream, tx: Sender<Envelope>) -> std::io::Result<()> {
    let (method, path, body) = read_request(&mut stream)?;
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

fn read_request(stream: &mut TcpStream) -> std::io::Result<(String, String, Vec<u8>)> {
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

    let content_length: usize = lines
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            if k.eq_ignore_ascii_case("content-length") {
                v.trim().parse().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    let mut body = buf[header_end..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(content_length.min(BODY_LIMIT));

    Ok((method, path, body))
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
    match (method, path) {
        ("GET", "/state") | ("GET", "/") => Req::State,
        ("GET", "/version") => Req::Version,
        ("GET", "/screenshot") | ("POST", "/screenshot") => Req::Screenshot,
        ("GET", "/ui_query") | ("POST", "/ui_query") => Req::UiQuery { label: label_param },
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
        ("POST", "/key") => Req::Key {
            key: json.get("key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            action: json.get("action").and_then(|v| v.as_str()).unwrap_or("tap").to_string(),
        },
        ("POST", "/ui_click") | ("POST", "/ui_hover") => Req::UiClick {
            label: label_param.unwrap_or_default(),
            action: json
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or(if path == "/ui_hover" { "hover" } else { "click" })
                .to_string(),
        },
        ("POST", "/mouse") => {
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
        _ => Req::Unknown(format!("{} {}", method, path)),
    }
}

// --- Response ---------------------------------------------------------------

fn reason_phrase(status: u16) -> &'static str {
    match status {
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
