//! Request dispatch, shared by the normal handler and `POST /batch`.
//!
//! [`ControlCtx`] owns everything a request may mutate; [`ControlCtx::handle`]
//! turns a [`Req`] into an [`Outcome`] without touching the world outside those
//! handles. Asynchronous requests (`/step`, `/screenshot`) are returned as
//! outcomes and completed by the caller, which also lets `/batch` reject them.

use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::http::{Req, Response};
use super::scene::ControlScene;
use super::state::{build_state, build_ui_query, version_body, ControlAction};
use super::{is_known_action, key_from_name, logs, text, ControlState, KtrlWorld};

/// Mutable state a request may touch, bundled as one `SystemParam`.
#[derive(SystemParam)]
pub(super) struct ControlCtx<'w, 's> {
    pub(super) keys: ResMut<'w, ButtonInput<KeyCode>>,
    pub(super) mouse_buttons: ResMut<'w, ButtonInput<MouseButton>>,
    pub(super) windows: Query<
        'w,
        's,
        (Entity, &'static mut Window),
        With<bevy::window::PrimaryWindow>,
    >,
    pub(super) commands: Commands<'w, 's>,
    pub(super) state: ResMut<'w, ControlState>,
    pub(super) virtual_time: ResMut<'w, Time<bevy::time::Virtual>>,
    pub(super) actions: MessageWriter<'w, ControlAction>,
    pub(super) keyboard: MessageWriter<'w, KeyboardInput>,
}

/// What handling a request produced.
pub(super) enum Outcome {
    Reply(Response),
    /// Needs the async screenshot observer (spawned by the caller).
    Screenshot { view: Option<String> },
    /// Needs the frame clock (applied by the caller).
    Step { frames: u32 },
}

fn from_result(result: Result<String, String>) -> Outcome {
    match result {
        Ok(body) => Outcome::Reply(Response::json(body)),
        Err(message) => Outcome::Reply(Response::text(404, message)),
    }
}

impl ControlCtx<'_, '_> {
    pub(super) fn handle(
        &mut self,
        req: Req,
        world: &mut KtrlWorld,
        scene: &ControlScene,
    ) -> Outcome {
        match req {
            Req::State => {
                let body = build_state(
                    &world.game_state,
                    &world.selection,
                    &world.project,
                    &world.editor_mode,
                    &self.state,
                    &world.diagnostics,
                    &world.extras,
                    &world.transforms,
                );
                Outcome::Reply(Response::json(body))
            }
            Req::Version => Outcome::Reply(Response::json(version_body(&self.state))),
            Req::Pause { on } => {
                self.state.paused = on;
                self.state.step_until = None;
                self.state.step_resp = None;
                if on {
                    self.virtual_time.pause();
                } else {
                    self.virtual_time.unpause();
                }
                let body = format!(
                    "{{\"ok\":true,\"paused\":{},\"frame\":{}}}",
                    on, self.state.frame
                );
                Outcome::Reply(Response::json(body))
            }
            Req::Step { frames } => Outcome::Step { frames },
            Req::Action { name, args } => {
                if is_known_action(&name) {
                    self.actions.write(ControlAction { name: name.clone(), args });
                    let body = format!(
                        "{{\"ok\":true,\"action\":\"{}\",\"frame\":{}}}",
                        name, self.state.frame
                    );
                    Outcome::Reply(Response::json(body))
                } else {
                    Outcome::Reply(Response::text(400, format!("unknown action: {name}")))
                }
            }
            Req::UiQuery { label } => {
                let mut current = std::collections::HashMap::new();
                for (entity, interaction) in world.interactions.iter() {
                    current.insert(entity, *interaction);
                }
                Outcome::Reply(Response::json(build_ui_query(&label, &world.ui_nodes, &current)))
            }
            Req::Logs { since, tail, level } => {
                Outcome::Reply(logs::respond(since, tail, level.as_deref()))
            }
            Req::Text { text } => {
                let window = self.windows.iter().next().map(|(entity, _)| entity);
                Outcome::Reply(text::respond(&text, window, &mut self.keyboard))
            }
            Req::Screenshot { view } => Outcome::Screenshot { view },
            Req::SceneTree { root, depth } => Outcome::Reply(Response::json(scene.tree(root, depth))),
            Req::EntityDetail { id } => from_result(scene.entity(id)),
            Req::MeshInfo { id } => from_result(scene.mesh(id)),
            Req::MaterialInfo { id } => from_result(scene.material(id)),
            Req::Key { key, action } => match key_from_name(&key) {
                Some(code) => {
                    match action.as_str() {
                        "press" => {
                            if !self.state.held.contains(&code) {
                                self.state.held.push(code);
                            }
                            self.keys.press(code);
                        }
                        "release" => {
                            self.state.held.retain(|c| *c != code);
                            self.keys.release(code);
                        }
                        _ => {
                            self.keys.press(code);
                            self.state.releases.push((code, 2));
                        }
                    }
                    Outcome::Reply(Response::json(format!("{{\"ok\":true,\"key\":\"{}\"}}", key)))
                }
                None => Outcome::Reply(Response::text(400, format!("unknown key: {key}"))),
            },
            Req::MouseMove { x, y } => {
                if let Ok((_, mut window)) = self.windows.single_mut() {
                    window.set_cursor_position(Some(Vec2::new(x, y)));
                }
                Outcome::Reply(Response::json("{\"ok\":true}".into()))
            }
            Req::UiClick { label, action } => {
                if action == "unhover" {
                    self.state.forced_hover = None;
                    return Outcome::Reply(Response::json("{\"ok\":true}".into()));
                }
                let mut found = None;
                for (entity, name) in world.names.iter() {
                    if name.as_str().trim().eq_ignore_ascii_case(label.trim()) {
                        found = Some(entity);
                        break;
                    }
                }
                if found.is_none() {
                    for (entity, text) in world.texts.iter() {
                        if text.0.trim().eq_ignore_ascii_case(label.trim()) {
                            found = Some(entity);
                            break;
                        }
                    }
                }
                let mut target = None;
                if let Some(mut current) = found {
                    for _ in 0..10 {
                        if world.interactions.get_mut(current).is_ok() {
                            target = Some(current);
                            break;
                        }
                        match world.parents.get(current) {
                            Ok(parent) => current = parent.0,
                            Err(_) => break,
                        }
                    }
                }
                let Some(entity) = target else {
                    return Outcome::Reply(Response::text(
                        404,
                        format!("no button with label: {label}"),
                    ));
                };
                if let Ok((_, mut interaction)) = world.interactions.get_mut(entity) {
                    if action == "hover" {
                        self.state.forced_hover = Some(entity);
                        *interaction = Interaction::Hovered;
                    } else {
                        *interaction = Interaction::Pressed;
                    }
                }
                let body = format!(
                    "{{\"ok\":true,\"entity\":{},\"action\":\"{}\"}}",
                    entity.index().index(),
                    action
                );
                Outcome::Reply(Response::json(body))
            }
            Req::MouseButton { button, action } => {
                let code = match button.as_str() {
                    "right" => MouseButton::Right,
                    "middle" => MouseButton::Middle,
                    _ => MouseButton::Left,
                };
                match action.as_str() {
                    "press" => self.mouse_buttons.press(code),
                    "release" => self.mouse_buttons.release(code),
                    _ => {
                        self.mouse_buttons.press(code);
                        self.mouse_buttons.release(code);
                    }
                }
                Outcome::Reply(Response::json("{\"ok\":true}".into()))
            }
            Req::Batch { .. } => Outcome::Reply(Response::text(
                400,
                "nested /batch is not supported".into(),
            )),
            Req::Unknown(path) => {
                Outcome::Reply(Response::text(404, format!("no such endpoint: {path}")))
            }
        }
    }
}
