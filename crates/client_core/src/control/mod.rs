//! KTRL — Klep2tron Control Server: local HTTP control API for agents
//! (screenshots, input, state, actions, events).
//!
//! Enabled by default in debug builds and via `settings.json`
//! (`"control": { "enabled": true, "port": 15703 }`) in release.
//! Binds to 127.0.0.1 only. API version: `ktrls/1`.
//! Roadmap: `plans/KTRL_Control_Server_Plan.md`.
//!
//! Layout: [`http`] is the transport/parser, [`dispatch`] turns requests into
//! responses, [`scene`]/[`state`] serialize the world, and this module owns the
//! ECS system that services requests.

use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::{EditorMode, GameState, Project, Selection};

mod batch;
mod config;
mod dispatch;
mod events;
mod http;
mod keys;
pub mod logs;
mod mutate;
mod scene;
mod state;
mod text;

use config::ControlConfig;
use dispatch::{ControlCtx, Outcome};
use http::{detect_binary, start_server, Envelope, Req, Response};
use keys::key_from_name;
use scene::ControlScene;
pub use state::{ControlAction, ControlExtras};
use state::publish_state_changes;

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
    mut ctx: ControlCtx,
    mut world: KtrlWorld,
    scene: ControlScene,
) {
    // Re-apply keys that the controller is holding down.
    for code in ctx.state.held.clone() {
        ctx.keys.press(code);
    }

    // Re-apply the forced UI hover (overrides the pointer-based value).
    if let Some(entity) = ctx.state.forced_hover {
        if let Ok((_, mut interaction)) = world.interactions.get_mut(entity) {
            *interaction = Interaction::Hovered;
        }
    }

    // Release keys whose hold expired.
    let mut index = 0;
    while index < ctx.state.releases.len() {
        if ctx.state.releases[index].1 == 0 {
            let code = ctx.state.releases[index].0;
            ctx.keys.release(code);
            ctx.state.releases.remove(index);
        } else {
            ctx.state.releases[index].1 -= 1;
            index += 1;
        }
    }

    // KTRL frame clock: counts every frame, even while virtual time is paused.
    ctx.state.frame = ctx.state.frame.wrapping_add(1);

    // Finish a pending `/step`: re-pause and release the held-back response.
    if ctx.state.step_until.is_some_and(|target| ctx.state.frame >= target) {
        ctx.state.step_until = None;
        if ctx.state.paused {
            ctx.virtual_time.pause();
        }
        if let Some(resp) = ctx.state.step_resp.take() {
            let body = format!(
                "{{\"ok\":true,\"frame\":{},\"paused\":{}}}",
                ctx.state.frame, ctx.state.paused
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
        // `/batch` runs its steps in order within this frame. `step`/`shot` are
        // rejected (the caller cannot block / go async mid-batch).
        if let Req::Batch { steps } = req {
            let mut results = Vec::with_capacity(steps.len());
            for step in steps {
                results.push(match ctx.handle(step, &mut world, &scene) {
                    Outcome::Reply(response) => serde_json::json!({
                        "status": response.status,
                        "body": String::from_utf8_lossy(&response.body),
                    }),
                    Outcome::Step { .. } | Outcome::Screenshot { .. } => serde_json::json!({
                        "status": 400,
                        "body": "step/screenshot are not supported in /batch",
                    }),
                });
            }
            let body = serde_json::json!({ "results": results });
            let _ = resp.send(Response::json(body.to_string()));
            continue;
        }

        match ctx.handle(req, &mut world, &scene) {
            Outcome::Reply(response) => {
                let _ = resp.send(response);
            }
            Outcome::Screenshot { view } => scene.capture(&mut ctx.commands, &view, resp),
            Outcome::Step { frames } => {
                if ctx.state.step_resp.is_some() {
                    let _ = resp.send(Response::text(409, "a step is already in progress".into()));
                } else {
                    ctx.virtual_time.unpause();
                    ctx.state.step_until = Some(ctx.state.frame.saturating_add(frames as u64));
                    ctx.state.step_resp = Some(resp);
                }
            }
        }
    }
}
