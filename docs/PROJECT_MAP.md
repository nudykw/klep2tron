# 🗺️ Klep2Tron Project Map

This document serves as a technical overview for AI assistance to quickly navigate and understand the codebase.

## 🏗️ Workspace Structure

- `crates/client`: Native game client binary.
- `crates/client_core`: Shared game logic and systems (The "Heart").
  - `src/rendering/`: Map drawing, materials, and mesh logic.
  - `src/ui/`: `menu`, `help`, and `hud` submodules.
  - `src/world.rs`: Environment, lighting, and camera setup.
  - `src/assets/`: Resource loading and progress bar.
  - `src/perf.rs`: Metrics collection and history.
  - `src/transition.rs`: Room switching logic and UI.
  - `src/input.rs`: Global shared controls (fullscreen).
  - `src/history.rs`: Undo/Redo stack management.
  - `GameState::ActorEditor` and `MenuAction::OpenActorEditor` are declared here, but the
    systems are provided by the separate `actor_editor` crate. The menu button is **not**
    hardcoded: binaries contribute it via `ExtraMenuButtons` `(label, action, tooltip)`.
- `crates/actor_editor`: Modular system for creating and editing NPC actors (native-only).
  - `ui/layout/`: Global layout, cameras, and lighting.
  - `ui/inspector/`: Detailed part and socket inspection.
  - `systems/`: Input, gizmos, slicing, and normalization logic.
  - `geometry/`: Mesh slicing, capping, and contour math.
  - Depends on `client_core` (only for `GameState` and `reset_ambient_light`) and `shared`.
  - **Not** a dependency of `client_web` / `editor_client_web` — it uses native file dialogs (`rfd`).
- `crates/client_web`: WASM wrapper for the game client.
- `crates/editor_client`: Native map editor.
  - `src/camera.rs`: Orbit controls and RTT sync.
  - `src/ui/`: Editor-specific buttons and tooltips.
  - `src/logic.rs`: Selection, mouse mapping, and history.
- `crates/editor_client_web`: WASM wrapper for the map editor.
- `crates/server`: Server-side logic (Docker/PostgreSQL).
- `crates/shared`: Shared data structures between client and server.
- `crates/admin_web`: Web interface for administration.
- `crates/ktrl`: KTRL client library (`ktrl::Client`) + `ktrl` CLI for the control API.

## 🔌 KTRL — Klep2tron Control Server (debug tooling)

Both native binaries start **KTRL** (Klep2tron Control Server), a local HTTP
control API, in debug builds (and in release when `settings.json` has
`"control": { "enabled": true }`), on `127.0.0.1:15703` (`KLEP_CONTROL`,
`KLEP_CONTROL_PORT` override it). API version `ktrls/1`.

- Source: `crates/client_core/src/control/` (`mod`, `http`, `state`, `scene`,
  `keys`, `config`).
- Client + CLI: `crates/ktrl` (`ktrl::Client` + `ktrl` binary, `cargo run -p
  ktrl -- ...`).
- Endpoints: `GET /state`, `GET /screenshot[?view=]`, `GET /version`,
  `GET /ui_query`, `GET /logs`, `GET /events` (SSE), `GET /scene_tree`,
  `GET /entity/{id}`, `GET /mesh/{id}`, `GET /material/{id}`, `POST /key`,
  `POST /text`, `POST /gamepad`, `POST /watch`, `GET /watch`,
  `POST /watch/clear`, `POST /mouse`, `POST /ui_click`,
  `POST /ui_hover`, `POST /pause`,
  `POST /step`, `POST /action`, `POST /batch`.
- Request dispatch lives in `control/dispatch.rs` (`ControlCtx::handle`);
  `POST /batch` runs a same-frame list of steps through it (`control/batch.rs`
  parses the steps).
- Logs: both binaries buffer the last 1000 `tracing` events (custom layer in
  `LogPlugin`), read back via `GET /logs[?since=&tail=&level=]`.
- Events: `GET /events` streams `log`/`state`/`panic` over SSE (fan-out in
  `control/events.rs`); panics are also recorded via a panic hook.
- Deterministic stepping: `POST /pause` freezes virtual time; `POST /step
  {"frames":N}` renders exactly N frames and blocks until done (no `sleep`).
- Offscreen capture: image-render-target cameras (`?view=camera:<id>`) are read
  back directly, so RTT thumbnails can be captured with the window occluded.
- Optional auth: `"control": { "token": "..." }` in `settings.json` or
  `KLEP_CONTROL_TOKEN`; then requests need `Authorization: Bearer <token>`.
  Bind address: `"control": { "bind": "..." }` / `KLEP_CONTROL_BIND` (default
  `127.0.0.1`); a token is mandatory for non-loopback binds. Remote access is
  possible directly (bind `0.0.0.0`/Tailscale IP) or via SSH/Tailscale tunnels.
- Cross-binary actions: `POST /action` emits `ControlAction` (handled by
  `client_core` for lifecycle + mutations, by `editor_client` for map edits).
  Editor-only state (`tool`, `undo`, `redo`) is merged into `/state` via
  `ControlExtras`.
- Mutations (`control/mutate.rs`): `SpawnEntity` (plain fixture entity, new id in
  `/state.last_spawned`), `DespawnEntity`, `SetTransform` (translation/scale/
  rotation, optional `relative`).
- Watchpoints (`control/watch.rs`): `POST /watch` predicates over `/state`
  fields or ECS entities; a hit softly pauses, emits `watch` on SSE with a state
  snapshot, and can save a screenshot (`watch_shot`). `GET /watch`,
  `POST /watch/clear`.
- Agent guide + helper script: `.agents/skills/klep2tron-control/`.
- Plan: [plans/KTRL_Control_Server_Plan.md](../plans/KTRL_Control_Server_Plan.md).
- Known issue: [plans/Preview_RTT_Thumbnails_Bug.md](../plans/Preview_RTT_Thumbnails_Bug.md).

## 🚦 State Machine (`GameState`)

Defined in `client_core/src/lib.rs`:
1. **Menu**: Main menu, initial landing.
2. **Loading**: Asset loading phase (meshes, textures).
3. **InGame**: Active gameplay or editor session.

## 📦 Key Resources

- **Project**: Holds the entire map data (Rooms, Cells).
- **ClientAssets**: Handles for meshes (`cube`, `wedge`) and materials.
- **TileMap**: Runtime cache of spawned entities mapped to coordinates.
- **DirtyTiles**: List of coordinates that need re-rendering (optimization).
- **PerfHistory**: In-memory storage for FPS, CPU, and RAM metrics.
- **CommandHistory**: Undo/Redo stack for the editor.

## 🎨 Rendering Logic (`map_rendering_system`)

Located in `client_core/src/rendering/mod.rs`.
- Uses **Partial Updates**: Only tiles in `DirtyTiles` are re-spawned.
- **Full Rebuild**: Triggered by room change or `dirty.full_rebuild`.
- **Change Detection**: Uses `project.is_changed()` to detect external loads.

## 🛠️ Editor Specifics

- **OrbitCamera**: Specialized 3D camera controller (`src/camera.rs`).
- **RTT Previews**: Render-to-Texture previews of tile types in the top panel.
- **Gizmos**: Used for selection highlights and dashed wedge outlines (`src/logic.rs`).

## 🚀 Common Commands

- **Run Editor**: `cargo run -p editor_client`
- **Run Game**: `cargo run -p client`
- **Check Workspace**: `cargo check --workspace`
