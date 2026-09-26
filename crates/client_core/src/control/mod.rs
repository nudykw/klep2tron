//! Local HTTP control API for agents (screenshots, input, state).
//!
//! Enabled by default in debug builds and via `settings.json`
//! (`"control": { "enabled": true, "port": 15703 }`) in release.
//! Binds to 127.0.0.1 only.

use bevy::prelude::*;
use bevy::render::view::screenshot::Screenshot;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::time::Duration;

use crate::{EditorMode, GameState, Project, Selection};

const BODY_LIMIT: usize = 1 << 20;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);

// --- Wire types -------------------------------------------------------------

pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

impl Response {
    fn json(body: String) -> Self {
        Self { status: 200, content_type: "application/json", body: body.into_bytes() }
    }
    fn text(status: u16, body: String) -> Self {
        Self { status, content_type: "text/plain; charset=utf-8", body: body.into_bytes() }
    }
    fn png(body: Vec<u8>) -> Self {
        Self { status: 200, content_type: "image/png", body }
    }
}

enum Req {
    State,
    Screenshot,
    Key { key: String, action: String },
    MouseMove { x: f32, y: f32 },
    MouseButton { button: String, action: String },
    /// `action`: `click` (one frame), `hover` (held), `unhover`.
    UiClick { label: String, action: String },
    Unknown(String),
}

struct Envelope {
    req: Req,
    resp: Sender<Response>,
}

#[derive(Resource)]
 struct ControlRx {
    rx: Mutex<Receiver<Envelope>>,
}

// --- Config -----------------------------------------------------------------

struct ControlConfig {
    enabled: bool,
    port: u16,
}

impl ControlConfig {
    fn load() -> Self {
        let mut enabled = cfg!(debug_assertions);
        let mut port = 15703u16;

        if let Ok(v) = std::env::var("KLEP_CONTROL") {
            enabled = v != "0" && !v.is_empty();
        }
        if let Ok(p) = std::env::var("KLEP_CONTROL_PORT") {
            if let Ok(p) = p.parse() {
                port = p;
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(content) = std::fs::read_to_string("settings.json") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(c) = json.get("control") {
                    if let Some(e) = c.get("enabled").and_then(|v| v.as_bool()) {
                        enabled = e;
                    }
                    if let Some(p) = c.get("port").and_then(|v| v.as_u64()) {
                        port = p as u16;
                    }
                }
            }
        }

        Self { enabled, port }
    }
}

// --- Plugin -----------------------------------------------------------------

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(target_arch = "wasm32") {
            return;
        }
        let cfg = ControlConfig::load();
        if !cfg.enabled {
            info!("control server: disabled");
            return;
        }
        match start_server(cfg.port) {
            Ok(rx) => {
                info!("control server: listening on http://127.0.0.1:{}", cfg.port);
                app.insert_resource(ControlRx { rx: Mutex::new(rx) });
                app.init_resource::<ControlState>();
                // Keep rendering even when the window is not focused, so that
                // screenshots always show a fresh frame instead of a stale/blank
                // swapchain image.
                app.insert_resource(bevy::winit::WinitSettings {
                    focused_mode: bevy::winit::UpdateMode::Continuous,
                    unfocused_mode: bevy::winit::UpdateMode::Continuous,
                });
                // `keyboard_input_system` clears `ButtonInput` every frame in
                // `PreUpdate` (InputSystems), so injected input must be applied
                // after that and before the game systems consume it.
                app.add_systems(
                    PreUpdate,
                    control_process_system
                        .after(bevy::input::InputSystems)
                        .after(bevy::ui::UiSystems::Focus),
                );
            }
            Err(e) => error!("control server: failed to bind port {}: {}", cfg.port, e),
        }
    }
}

fn start_server(port: u16) -> std::io::Result<Receiver<Envelope>> {
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

// --- Minimal HTTP -----------------------------------------------------------

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

fn parse_request(method: &str, path: &str, body: &[u8]) -> Req {
    let path = path.split('?').next().unwrap_or(path);
    let json: serde_json::Value = serde_json::from_slice(body).unwrap_or(serde_json::Value::Null);
    match (method, path) {
        ("GET", "/state") | ("GET", "/") => Req::State,
        ("GET", "/screenshot") | ("POST", "/screenshot") => Req::Screenshot,
        ("POST", "/key") => Req::Key {
            key: json.get("key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            action: json.get("action").and_then(|v| v.as_str()).unwrap_or("tap").to_string(),
        },
        ("POST", "/ui_click") | ("POST", "/ui_hover") => Req::UiClick {
            label: json.get("label").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
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

fn write_response(stream: &mut TcpStream, resp: &Response) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        resp.status, resp.content_type, resp.body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&resp.body)?;
    stream.flush()
}

// --- ECS side ---------------------------------------------------------------

fn key_from_name(name: &str) -> Option<KeyCode> {
    use KeyCode::*;
    Some(match name {
        "ArrowUp" | "Up" => ArrowUp,
        "ArrowDown" | "Down" => ArrowDown,
        "ArrowLeft" | "Left" => ArrowLeft,
        "ArrowRight" | "Right" => ArrowRight,
        "Enter" | "Return" => Enter,
        "Space" => Space,
        "Escape" | "Esc" => Escape,
        "Tab" => Tab,
        "Backspace" => Backspace,
        "Delete" => Delete,
        "ShiftLeft" | "Shift" => ShiftLeft,
        "ControlLeft" | "Ctrl" => ControlLeft,
        "AltLeft" | "Alt" => AltLeft,
        "F1" => F1,
        "F2" => F2,
        "F3" => F3,
        "F4" => F4,
        "F5" => F5,
        "KeyA" => KeyA,
        "KeyB" => KeyB,
        "KeyC" => KeyC,
        "KeyD" => KeyD,
        "KeyE" => KeyE,
        "KeyF" => KeyF,
        "KeyG" => KeyG,
        "KeyH" => KeyH,
        "KeyI" => KeyI,
        "KeyJ" => KeyJ,
        "KeyK" => KeyK,
        "KeyL" => KeyL,
        "KeyM" => KeyM,
        "KeyN" => KeyN,
        "KeyO" => KeyO,
        "KeyP" => KeyP,
        "KeyQ" => KeyQ,
        "KeyR" => KeyR,
        "KeyS" => KeyS,
        "KeyT" => KeyT,
        "KeyU" => KeyU,
        "KeyV" => KeyV,
        "KeyW" => KeyW,
        "KeyX" => KeyX,
        "KeyY" => KeyY,
        "KeyZ" => KeyZ,
        "Digit0" => Digit0,
        "Digit1" => Digit1,
        "Digit2" => Digit2,
        "Digit3" => Digit3,
        "Digit4" => Digit4,
        "Digit5" => Digit5,
        "Digit6" => Digit6,
        "Digit7" => Digit7,
        "Digit8" => Digit8,
        "Digit9" => Digit9,
        _ => {
            if name.len() == 1 {
                let c = name.chars().next().unwrap();
                if c.is_ascii_alphabetic() {
                    // fall through to the match above is not possible; map here
                    return key_from_char(c);
                }
                if c.is_ascii_digit() {
                    return key_from_digit(c);
                }
            }
            return None;
        }
    })
}

fn key_from_char(c: char) -> Option<KeyCode> {
    use KeyCode::*;
    Some(match c.to_ascii_uppercase() {
        'A' => KeyA, 'B' => KeyB, 'C' => KeyC, 'D' => KeyD, 'E' => KeyE, 'F' => KeyF,
        'G' => KeyG, 'H' => KeyH, 'I' => KeyI, 'J' => KeyJ, 'K' => KeyK, 'L' => KeyL,
        'M' => KeyM, 'N' => KeyN, 'O' => KeyO, 'P' => KeyP, 'Q' => KeyQ, 'R' => KeyR,
        'S' => KeyS, 'T' => KeyT, 'U' => KeyU, 'V' => KeyV, 'W' => KeyW, 'X' => KeyX,
        'Y' => KeyY, 'Z' => KeyZ,
        _ => return None,
    })
}

fn key_from_digit(c: char) -> Option<KeyCode> {
    use KeyCode::*;
    Some(match c {
        '0' => Digit0, '1' => Digit1, '2' => Digit2, '3' => Digit3, '4' => Digit4,
        '5' => Digit5, '6' => Digit6, '7' => Digit7, '8' => Digit8, '9' => Digit9,
        _ => return None,
    })
}

/// Controller input state that must be re-applied every frame.
#[derive(Resource, Default)]
struct ControlState {
    /// Keys released after a couple of frames, so `just_released` fires.
    releases: Vec<(KeyCode, u32)>,
    /// Keys held down; re-applied every frame because Bevy clears `ButtonInput`.
    held: Vec<KeyCode>,
    /// A UI entity kept hovered; re-applied every frame because `ui_focus_system`
    /// recomputes `Interaction` from the real pointer.
    forced_hover: Option<Entity>,
}

#[allow(clippy::too_many_arguments)]
fn control_process_system(
    rx: Option<Res<ControlRx>>,
    game_state: Option<Res<State<GameState>>>,
    selection: Option<Res<Selection>>,
    project: Option<Res<Project>>,
    editor_mode: Option<Res<EditorMode>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse_buttons: ResMut<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    transforms: Query<(Entity, &Transform)>,
    mut commands: Commands,
    mut state: ResMut<ControlState>,
    texts: Query<(Entity, &Text)>,
    names: Query<(Entity, &Name)>,
    parents: Query<&ChildOf>,
    mut interactions: Query<&mut Interaction>,
) {
    // Re-apply keys that the controller is holding down.
    for code in state.held.clone() {
        keys.press(code);
    }

    // Re-apply the forced UI hover (overrides the pointer-based value).
    if let Some(entity) = state.forced_hover {
        if let Ok(mut interaction) = interactions.get_mut(entity) {
            *interaction = Interaction::Hovered;
        }
    }

    // Release keys whose hold expired.
    state.releases.retain_mut(|(code, frames)| {
        if *frames == 0 {
            keys.release(*code);
            false
        } else {
            *frames -= 1;
            true
        }
    });

    let Some(rx) = rx else { return };

    let requests: Vec<Envelope> = {
        let lock = rx.rx.lock().unwrap();
        lock.try_iter().collect()
    };

    for Envelope { req, resp } in requests {
        match req {
            Req::State => {
                let state = build_state(&game_state, &selection, &project, &editor_mode, &transforms);
                let _ = resp.send(Response::json(state));
            }
            Req::Screenshot => {
                let resp = resp.clone();
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(move |captured: On<bevy::render::view::screenshot::ScreenshotCaptured>| {
                        let image = captured.image.clone();
                        let body = match image.try_into_dynamic() {
                            Ok(dyn_img) => {
                                let mut buf = std::io::Cursor::new(Vec::new());
                                match dyn_img.write_to(&mut buf, image::ImageFormat::Png) {
                                    Ok(()) => Response::png(buf.into_inner()),
                                    Err(e) => Response::text(500, format!("png encode failed: {e}")),
                                }
                            }
                            Err(e) => Response::text(500, format!("image convert failed: {e}")),
                        };
                        let _ = resp.send(body);
                    });
            }
            Req::Key { key, action } => {
                match key_from_name(&key) {
                    Some(code) => {
                        match action.as_str() {
                            "press" => {
                                if !state.held.contains(&code) {
                                    state.held.push(code);
                                }
                                keys.press(code);
                            }
                            "release" => {
                                state.held.retain(|c| *c != code);
                                keys.release(code);
                            }
                            _ => {
                                keys.press(code);
                                state.releases.push((code, 2));
                            }
                        }
                        let _ = resp.send(Response::json(format!("{{\"ok\":true,\"key\":\"{}\"}}", key)));
                    }
                    None => {
                        let _ = resp.send(Response::text(400, format!("unknown key: {key}")));
                    }
                }
            }
            Req::MouseMove { x, y } => {
                if let Ok(mut window) = windows.single_mut() {
                    window.set_cursor_position(Some(Vec2::new(x, y)));
                }
                let _ = resp.send(Response::json("{\"ok\":true}".into()));
            }
            Req::UiClick { label, action } => {
                match action.as_str() {
                    "unhover" => {
                        state.forced_hover = None;
                        let _ = resp.send(Response::json("{\"ok\":true}".into()));
                    }
                    _ => {
                        let mut found = None;
                        for (entity, name) in names.iter() {
                            if name.as_str().trim().eq_ignore_ascii_case(label.trim()) {
                                found = Some(entity);
                                break;
                            }
                        }
                        if found.is_none() {
                            for (entity, text) in texts.iter() {
                                if text.0.trim().eq_ignore_ascii_case(label.trim()) {
                                    found = Some(entity);
                                    break;
                                }
                            }
                        }
                        let mut target = None;
                        if let Some(mut current) = found {
                            for _ in 0..10 {
                                if interactions.get_mut(current).is_ok() {
                                    target = Some(current);
                                    break;
                                }
                                match parents.get(current) {
                                    Ok(parent) => current = parent.0,
                                    Err(_) => break,
                                }
                            }
                        }
                        match target {
                            Some(entity) => {
                                if let Ok(mut interaction) = interactions.get_mut(entity) {
                                    if action == "hover" {
                                        state.forced_hover = Some(entity);
                                        *interaction = Interaction::Hovered;
                                    } else {
                                        *interaction = Interaction::Pressed;
                                    }
                                }
                                let _ = resp.send(Response::json(format!(
                                    "{{\"ok\":true,\"entity\":{},\"action\":\"{}\"}}",
                                    entity.index(),
                                    action
                                )));
                            }
                            None => {
                                let _ = resp.send(Response::text(
                                    404,
                                    format!("no button with label: {label}"),
                                ));
                            }
                        }
                    }
                }
            }
            Req::MouseButton { button, action } => {
                let code = match button.as_str() {
                    "right" => MouseButton::Right,
                    "middle" => MouseButton::Middle,
                    _ => MouseButton::Left,
                };
                match action.as_str() {
                    "press" => mouse_buttons.press(code),
                    "release" => mouse_buttons.release(code),
                    _ => {
                        mouse_buttons.press(code);
                        mouse_buttons.release(code);
                    }
                }
                let _ = resp.send(Response::json("{\"ok\":true}".into()));
            }
            Req::Unknown(path) => {
                let _ = resp.send(Response::text(404, format!("no such endpoint: {path}")));
            }
        }
    }
}

fn build_state(
    game_state: &Option<Res<State<GameState>>>,
    selection: &Option<Res<Selection>>,
    project: &Option<Res<Project>>,
    editor_mode: &Option<Res<EditorMode>>,
    transforms: &Query<(Entity, &Transform)>,
) -> String {
    let gs = game_state
        .as_ref()
        .map(|s| format!("{:?}", s.get()))
        .unwrap_or_else(|| "unknown".into());

    let mut entities = Vec::new();
    for (entity, transform) in transforms.iter().take(4096) {
        let t = transform.translation;
        entities.push(format!(
            "{{\"entity\":{},\"pos\":[{:.3},{:.3},{:.3}]}}",
            entity.index(),
            t.x, t.y, t.z
        ));
    }

    let selection_json = match selection {
        Some(s) => format!("{{\"x\":{},\"z\":{}}}", s.x, s.z),
        None => "null".into(),
    };
    let editor_active = editor_mode.as_ref().map(|m| m.is_active).unwrap_or(false);

    let map_json = match project {
        Some(p) if !p.rooms.is_empty() => {
            let room = &p.rooms[p.current_room_idx];
            let mut cells = String::from("[");
            for x in 0..16 {
                cells.push('[');
                for z in 0..16 {
                    if z > 0 {
                        cells.push(',');
                    }
                    let c = &room.cells[x][z];
                    cells.push_str(&format!("{{\"h\":{},\"tt\":\"{:?}\"}}", c.h, c.tt));
                }
                cells.push(']');
                if x < 15 {
                    cells.push(',');
                }
            }
            cells.push(']');
            format!(
                "{{\"current_room\":{},\"rooms\":{},\"cells\":{}}}",
                p.current_room_idx,
                p.rooms.len(),
                cells
            )
        }
        _ => "null".into(),
    };

    format!(
        "{{\"game_state\":\"{}\",\"editor_active\":{},\"selection\":{},\"map\":{},\"entities\":[{}]}}",
        gs,
        editor_active,
        selection_json,
        map_json,
        entities.join(",")
    )
}
