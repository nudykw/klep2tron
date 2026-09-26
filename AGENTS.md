# AGENTS.md — Klep2tron

Klep2tron is a cross-platform 2.5D client-server game in Rust on **Bevy 0.19.x**,
plus a level editor and an NPC/actor editor. It is a Cargo workspace. Primary
development happens on macOS (Apple Silicon); Linux and Web (WASM) targets are
supported.

> The user-level `~/AGENTS.md` describes the `aorus-cachyos-server` LLM inference
> host (vLLM/ComfyUI/GPUs). That is a different machine and has nothing to do
> with this repository — this file applies to Klep2tron work.

## Golden rules

1. **Plan first.** Anything beyond a typo starts with a plan in `plans/`
   (templates in `docs/templates/`). Follow `docs/PRODUCT_PROTOCOL.md` and
   `docs/ENGINEERING_PROTOCOL.md` for non-trivial features.
2. **Never run `git commit`, `git push`, or mutating `gh` commands without an
   explicit user command** (`docs/AI_WORKFLOW.md` §2). Editing the working tree
   is fine; syncing the repository is a manual barrier.
3. **Always compile before declaring done:** `cargo check --workspace` (or the
   relevant `-p <crate>`), plus `cargo test --workspace --no-run`.
4. **Update the docs.** If behavior or commands change, update the matching doc.
   Keep the EN/UA pairs in sync (`docs/*_UA.md` ↔ `docs/*.md`, `README.md`).
5. **Follow SOLID for Bevy ECS** (`docs/SOLID_BEVY.md`): plugins over
   monoliths, events/`NextState` over direct system calls, tiny components,
   precise query filters, no dead code, files ≤ 500 lines.

## Commands

```bash
cargo check --workspace
cargo build --workspace
cargo test --workspace --no-run          # tests are sparse; this compiles them

cargo run -p editor_client               # native map editor
cargo run -p client                      # native game client
cargo run -p server                      # authoritative server (needs PostgreSQL)
```

Web (WASM) via [Trunk](https://trunkrs.dev/):
```bash
cd crates/client_web && trunk serve         # :8081
cd crates/editor_client_web && trunk serve  # :8082
```

Running a built binary directly (not through `cargo run`) needs the asset root,
otherwise Bevy looks in `target/debug/assets`:
```bash
BEVY_ASSET_ROOT=$PWD ./target/debug/editor_client
```

## Workspace layout

| Crate | Role | Targets |
|---|---|---|
| `shared` | Map/tile/protocol data types | all |
| `client_core` | Shared game logic + systems; UI, rendering, control API | native + wasm |
| `client` | Native game client binary | native |
| `editor_client` | Native map editor (orbit camera, RTT tile previews) | native |
| `actor_editor` | NPC/actor editor: slicing, sockets, gizmos (uses `rfd`) | **native only** |
| `client_web` | WASM wrapper for the game client | wasm |
| `editor_client_web` | WASM wrapper for the map editor | wasm |
| `server` | Authoritative game server (Docker/PostgreSQL) | native |
| `admin_web` | Server admin panel (Axum) | native |
| `ktrl` | Client + CLI for the KTRL control API (`ureq`, `clap`) | native |

`actor_editor` is deliberately **not** a dependency of the web crates.

## Where to look

- `docs/PROJECT_MAP.md` — module-by-module map (start here).
- `docs/EDITOR_GUIDE.md` — map editor controls.
- `docs/ROOM_RENDERING.md`, `docs/Actor_Storage_Format.md` — subsystems.
- `docs/design/GDD.md` — game design.
- `plans/` — active plans and post-mortems. Relevant right now:
  `plans/Bevy019_Migration_Plan.md`, `plans/Preview_RTT_Thumbnails_Bug.md`.
- `.agents/skills/` — on-demand workflows (`klep2tron-control`,
  `klep2tron-build`, `klep2tron-docs`, `bevy-upgrade`).

## Debug tooling: control API

Debug builds of `client` / `editor_client` expose a local HTTP API on
`http://127.0.0.1:15703` (`/state`, `/screenshot`, `/key`, `/mouse`,
`/ui_click`, `/ui_hover`). Use it to verify rendering/UI changes with
screenshots instead of guessing. See the `klep2tron-control` skill.

## Platform and dependency notes

- Native crates: `bevy = { version = "0.19", features = ["default", "wayland"] }`;
  web crates: `features = ["webgl2"]`.
- Linux needs Bevy's system libs (`libx11`, `libasound2`, `libudev`, `libwayland`,
  `libxkbcommon`, …); macOS needs Xcode Command Line Tools. Details in `README.md`.
- Release profile: `lto = true`, `opt-level = 's'`, `panic = "abort"`.
- macOS linker warning `__eh_frame section too large ...` is benign.

## Bevy 0.19 traps (learned the hard way)

- **Never put `GlobalZIndex` on every UI node.** In `bevy_ui` the *presence* of
  `GlobalZIndex` (even `GlobalZIndex(0)`) makes the node a separate stacking
  root, so draw order falls back to archetype order and an unrelated archetype
  move can reorder siblings. This caused the disappearing editor thumbnails.
  `client_core::ui::widgets::UiNode` intentionally has no `GlobalZIndex` field;
  add `GlobalZIndex(n)` as a separate component only for real overlays.
  Root cause: Bevy [#25410](https://github.com/bevyengine/bevy/issues/25410),
  fixed by [#25441](https://github.com/bevyengine/bevy/pull/25441) (0.19.2).
- The editor's RTT tile previews must keep rotating with the orbit camera
  (`sync_rtt_cameras_system` / `RttCameraTarget` in `crates/editor_client`).
- If a bug turns out to be in Bevy, make a minimal repro and link the upstream
  issue/PR instead of shipping a workaround. See the `bevy-upgrade` skill.
