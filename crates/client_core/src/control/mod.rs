//! KTRL — Klep2tron Control Server: local HTTP control API for agents
//! (screenshots, input, state, actions).
//!
//! Enabled by default in debug builds and via `settings.json`
//! (`"control": { "enabled": true, "port": 15703 }`) in release.
//! Binds to 127.0.0.1 only. API version: `ktrls/1`.
//! Roadmap: `plans/KTRL_Control_Server_Plan.md`.
//!
//! Layout: [`http`] is the transport/parser, [`state`] the serialization and
//! host-facing types, this module the ECS system that services requests.

use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::{EditorMode, GameState, Project, Selection};

mod config;
mod events;
mod http;
mod keys;
pub mod logs;
mod mutate;
mod scene;
mod state;
mod text;

use config::ControlConfig;
use http::{detect_binary, start_server, Envelope, Req, Response};
use keys::key_from_name;
use scene::ControlScene;
pub use state::{ControlAction, ControlExtras};
use state::{build_state, build_ui_query, publish_state_changes};

const BODY_LIMIT: usize = 1 << 20;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
/// Upper bound on `/step` frames: the HTTP thread gives up after
/// [`RESPONSE_TIMEOUT`], so a request must finish well within 10 s.
const MAX_STEP_FRAMES: u32 = 120;

/// Actions accepted by `POST /action`.
const KNOWN_ACTIONS: &[&str] = &[
    "StartGame",
    "StartEditor",
    "QuitToMenu",
    "Exit",
    "SetSelection",
    "SetTile",
    "SetTileType",
    "Undo",
    "Redo",
    "NextRoom",
    "PrevRoom",
    "AddRoom",
    "ClearRoom",
    "SaveMap",
    "LoadMap",
    "SetTransform", "DespawnEntity", "SpawnEntity",
];

fn is_known_action(name: &str) -> bool {
    KNOWN_ACTIONS.contains(&name)
}

#[derive(Resource)]
struct ControlRx {
    rx: Mutex<Receiver<Envelope>>,
}

// --- Plugin -----------------------------------------------------------------

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(target_arch = "wasm32") {
            return;
        }
        // Registered even when the server is off, so host binaries can add
        // their own `ControlAction` handlers unconditionally.
        app.add_message::<ControlAction>();

        let cfg = ControlConfig::load();
        if !cfg.enabled {
            info!("KTRL: disabled");
            return;
        }
        match start_server(cfg.port, cfg.token.as_deref().map(Arc::<str>::from)) {
            Ok(rx) => {
                info!("KTRL: listening on http://127.0.0.1:{}", cfg.port);
                app.insert_resource(ControlRx { rx: Mutex::new(rx) });
                app.init_resource::<ControlState>();
                app.init_resource::<ControlExtras>();
                {
                    let mut state = app.world_mut().resource_mut::<ControlState>();
                    state.binary = detect_binary();
                    state.port = cfg.port;
                }
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
                    (
                        control_process_system
                            .after(bevy::input::InputSystems)
                            .after(bevy::ui::UiSystems::Focus),
                        handle_core_control_actions.after(control_process_system),
                        publish_state_changes.after(control_process_system),
                        mutate::handle_mutations.after(control_process_system),
                    ),
                );
            }
            Err(e) => error!("KTRL: failed to bind port {}: {}", cfg.port, e),
        }
    }
}

/// Core lifecycle actions shared by every host binary.
fn handle_core_control_actions(
    mut actions: MessageReader<ControlAction>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut editor_mode: ResMut<EditorMode>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    for action in actions.read() {
        match action.name.as_str() {
            "StartGame" => {
                next_game_state.set(GameState::Loading);
                editor_mode.is_active = false;
            }
            "StartEditor" => {
                next_game_state.set(GameState::Loading);
                editor_mode.is_active = true;
            }
            "QuitToMenu" => next_game_state.set(GameState::Menu),
            "Exit" => {
                exit.write(bevy::app::AppExit::Success);
            }
            _ => {}
        }
    }
}

// --- ECS side ---------------------------------------------------------------

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
    /// Monotonic frame counter, incremented even while virtual time is paused.
    frame: u64,
    /// `true` while `/pause` froze virtual time (the frame clock keeps running).
    paused: bool,
    /// A `/step` is running until the frame clock reaches this value.
    step_until: Option<u64>,
    /// Response held back until a `/step` completes, so it returns the final frame.
    step_resp: Option<Sender<Response>>,
    /// Binary name reported by `/version`.
    binary: String,
    /// Bound port reported by `/version`.
    port: u16,
}

/// Read-only world view used by `/state` and `/ui_query`, bundled into one
/// `SystemParam` to stay within Bevy's per-system parameter limit.
#[derive(SystemParam)]
struct KtrlWorld<'w, 's> {
    game_state: Option<Res<'w, State<GameState>>>,
    selection: Option<Res<'w, Selection>>,
    project: Option<Res<'w, Project>>,
    editor_mode: Option<Res<'w, EditorMode>>,
    transforms: Query<'w, 's, (Entity, &'static Transform, Option<&'static Name>)>,
    ui_nodes: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static Name>,
            Option<&'static Text>,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    texts: Query<'w, 's, (Entity, &'static Text)>,
    names: Query<'w, 's, (Entity, &'static Name)>,
    parents: Query<'w, 's, &'static ChildOf>,
    interactions: Query<'w, 's, (Entity, &'static mut Interaction)>,
    diagnostics: Res<'w, DiagnosticsStore>,
    extras: Option<Res<'w, ControlExtras>>,
}

fn control_process_system(
    rx: Option<Res<ControlRx>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse_buttons: ResMut<ButtonInput<MouseButton>>,
    mut windows: Query<(Entity, &mut Window), With<bevy::window::PrimaryWindow>>,
    mut commands: Commands,
    mut state: ResMut<ControlState>,
    mut virtual_time: ResMut<Time<bevy::time::Virtual>>,
    mut actions: MessageWriter<ControlAction>,
    mut keyboard: MessageWriter<KeyboardInput>,
    world: KtrlWorld,
    scene: ControlScene,
) {
    let KtrlWorld {
        game_state,
        selection,
        project,
        editor_mode,
        transforms,
        ui_nodes,
        texts,
        names,
        parents,
        mut interactions,
        diagnostics,
        extras,
    } = world;

    // Re-apply keys that the controller is holding down.
    for code in state.held.clone() {
        keys.press(code);
    }

    // Re-apply the forced UI hover (overrides the pointer-based value).
    if let Some(entity) = state.forced_hover {
        if let Ok((_, mut interaction)) = interactions.get_mut(entity) {
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

    // KTRL frame clock: counts every frame, even while virtual time is paused.
    state.frame = state.frame.wrapping_add(1);

    // Finish a pending `/step`: re-pause and release the held-back response.
    if state.step_until.is_some_and(|target| state.frame >= target) {
        state.step_until = None;
        if state.paused {
            virtual_time.pause();
        }
        if let Some(resp) = state.step_resp.take() {
            let body = format!(
                "{{\"ok\":true,\"frame\":{},\"paused\":{}}}",
                state.frame, state.paused
            );
            let _ = resp.send(Response::json(body));
        }
    }

    let Some(rx) = rx else { return };

    let requests: Vec<Envelope> = {
        let lock = rx.rx.lock().unwrap();
        lock.try_iter().collect()
    };

    for Envelope { req, resp } in requests {
        match req {
            Req::State => {
                let body = build_state(
                    &game_state,
                    &selection,
                    &project,
                    &editor_mode,
                    &state,
                    &diagnostics,
                    &extras,
                    &transforms,
                );
                let _ = resp.send(Response::json(body));
            }
            Req::Version => {
                let _ = resp.send(Response::json(state::version_body(&state)));
            }
            Req::Pause { on } => {
                state.paused = on;
                state.step_until = None;
                state.step_resp = None;
                if on {
                    virtual_time.pause();
                } else {
                    virtual_time.unpause();
                }
                let body = format!(
                    "{{\"ok\":true,\"paused\":{},\"frame\":{}}}",
                    on, state.frame
                );
                let _ = resp.send(Response::json(body));
            }
            Req::Step { frames } => {
                if state.step_resp.is_some() {
                    let _ = resp.send(Response::text(409, "a step is already in progress".into()));
                } else {
                    virtual_time.unpause();
                    state.step_until = Some(state.frame.saturating_add(frames as u64));
                    state.step_resp = Some(resp);
                }
            }
            Req::Action { name, args } => {
                if is_known_action(&name) {
                    actions.write(ControlAction { name: name.clone(), args });
                    let body = format!(
                        "{{\"ok\":true,\"action\":\"{}\",\"frame\":{}}}",
                        name, state.frame
                    );
                    let _ = resp.send(Response::json(body));
                } else {
                    let _ = resp.send(Response::text(400, format!("unknown action: {name}")));
                }
            }
            Req::UiQuery { label } => {
                let mut current = std::collections::HashMap::new();
                for (entity, interaction) in interactions.iter() {
                    current.insert(entity, *interaction);
                }
                let _ = resp.send(Response::json(build_ui_query(&label, &ui_nodes, &current)));
            }
            Req::Logs { since, tail, level } => {
                let _ = resp.send(logs::respond(since, tail, level.as_deref()));
            }
            Req::Text { text } => {
                let window = windows.iter().next().map(|(entity, _)| entity);
                let _ = resp.send(text::respond(&text, window, &mut keyboard));
            }
            Req::Screenshot { view } => {
                scene.capture(&mut commands, &view, resp);
            }
            Req::SceneTree { root, depth } => {
                let _ = resp.send(Response::json(scene.tree(root, depth)));
            }
            Req::EntityDetail { id } => match scene.entity(id) {
                Ok(body) => {
                    let _ = resp.send(Response::json(body));
                }
                Err(msg) => {
                    let _ = resp.send(Response::text(404, msg));
                }
            },
            Req::MeshInfo { id } => match scene.mesh(id) {
                Ok(body) => {
                    let _ = resp.send(Response::json(body));
                }
                Err(msg) => {
                    let _ = resp.send(Response::text(404, msg));
                }
            },
            Req::MaterialInfo { id } => match scene.material(id) {
                Ok(body) => {
                    let _ = resp.send(Response::json(body));
                }
                Err(msg) => {
                    let _ = resp.send(Response::text(404, msg));
                }
            },
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
                if let Ok((_, mut window)) = windows.single_mut() {
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
                                if let Ok((_, mut interaction)) = interactions.get_mut(entity) {
                                    if action == "hover" {
                                        state.forced_hover = Some(entity);
                                        *interaction = Interaction::Hovered;
                                    } else {
                                        *interaction = Interaction::Pressed;
                                    }
                                }
                                let _ = resp.send(Response::json(format!(
                                    "{{\"ok\":true,\"entity\":{},\"action\":\"{}\"}}",
                                    entity.index().index(),
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

