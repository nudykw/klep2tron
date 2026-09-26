---
name: bevy-upgrade
description: Upgrade Klep2tron to a newer Bevy release or adapt code to Bevy API changes, including fixing post-migration compile errors and diagnosing rendering/UI regressions that come from Bevy itself. Use when bumping bevy/bevy_* crates, migrating after an engine release, or when a bug looks like a Bevy defect.
---

# Bevy upgrades

Klep2tron currently pins **Bevy 0.19.1** (`Cargo.lock` is committed). Ecosystem
crates: `bevy_obj 0.19`, `bevy_panorbit_camera 0.35`, `bevy_hanabi 0.19`,
`bevy_framepace 0.22`. `wgpu` follows Bevy.

The 0.14 → 0.19 migration is documented in `plans/Bevy019_Migration_Plan.md`:
~500 direct edits across ~23 unique symbols (≈90% mechanical). Read it before a
major bump.

## Workflow

1. **Read every skipped migration guide.** The official Bevy migration guide is
   per release; there is no single jump guide. For 0.14 → 0.19 that means
   0.15, 0.16, …, 0.19. Take the API names from the guides of the moment, not
   from memory.
2. **Update manifests first** (`Cargo.toml` in every crate), keep `Cargo.lock`
   in sync, and make sure `wgpu` is not duplicated.
3. **Compile crate by crate** in dependency order:
   `shared` → `server` → `client_core` → `actor_editor` → `client` /
   `editor_client` → web wrappers.
   `cargo check -p <crate>` and fix, before moving on.
4. **Split mechanical vs semantic changes.** Mechanical renames can be batched;
   give the semantic areas their own pass and their own smoke test:
   render (SSAO/bloom/fog/custom `Material`/`ShaderRef`), picking, UI required
   components, asset pipeline.
5. **Smoke after each stage**:
   - `cargo check --workspace`
   - `cargo check -p client_web --target wasm32-unknown-unknown`
   - `cargo check -p editor_client_web --target wasm32-unknown-unknown`
   - run `client` (menu → game) and `editor_client` (UI, RTT previews,
     orbit camera); use the `klep2tron-control` skill.
6. **Update docs**: `docs/PROJECT_MAP.md`, `README.md`, and any subsystem doc
   that mentions the old API/version. Keep the EN/UA pairs in sync
   (`klep2tron-docs` skill).

## Recurring rename cheat sheet (0.14 → 0.19)

| Old | New |
|---|---|
| `Style` | `Node` |
| `TextStyle` | `TextFont` + `TextColor` |
| `NodeBundle` / `TextBundle` / `PbrBundle` / `*Bundle` | components directly (required components) |
| `Query::get_single` / `get_single_mut` | `single` / `single_mut` (return `Result`) |
| `despawn_recursive()` | `despawn()` (cascade by default) |
| `Parent` | `ChildOf` |
| `ChildBuilder` | `ChildSpawnerCommands` |
| `Time::delta_seconds` / `elapsed_seconds` | `delta_secs` / `elapsed_secs` |
| `bevy_mod_picking::*` | built-in `bevy_picking` (dead crate; remove the dep) |
| `GamepadButtonType`, `JustifyText`, `FogSettings`, `apply_deferred`, `NotShadowCaster/Receiver`, `MaterialMeshBundle` | renamed / restructured |
| `bevy::core_pipeline::bloom`, `ScreenSpaceAmbientOcclusion*`, `ShaderRef` | moved/renamed render paths |

This table is a starting point, **not** a substitute for the per-release guides.

## Traps specific to this project

- **`UiNode` and `GlobalZIndex`.** Never re-add a default `GlobalZIndex` to the
  `client_core::ui::widgets::UiNode` bundle: its presence (even `0`) makes the
  node a separate stacking root and draw order becomes archetype-dependent
  (Bevy [#25410](https://github.com/bevyengine/bevy/issues/25410), fixed by
  [#25441](https://github.com/bevyengine/bevy/pull/25441) in 0.19.2). See
  `plans/Preview_RTT_Thumbnails_Bug.md`.
- **RTT editor previews.** `sync_rtt_cameras_system` / `RttCameraTarget` must
  keep the offscreen cameras orbiting with the main camera. Verify with
  `klep2tron-control` after any render/camera change.
- **Features differ by target:** native uses `features = ["default", "wayland"]`,
  web uses `features = ["webgl2"]`. `actor_editor` is native-only (`rfd`).
- **Assets in WASM** live per-crate under `crates/*_web/assets`; native runs
  need `BEVY_ASSET_ROOT=$PWD` when the binary is run directly.

## When the bug is in Bevy

1. Make a **minimal repro** that uses only public Bevy APIs (a tiny app is
   better than a patched engine).
2. Search existing issues first: <https://github.com/bevyengine/bevy/issues>.
3. If it is new, file it with the version, platform, repro, and expected vs
   actual behavior, then link it from the relevant `plans/*.md`.
4. Prefer applying an upstream fix / bumping to the fixed release over a local
   workaround. If a workaround is unavoidable, document the upstream link and
   the condition for removing it.

## Verification checklist

- [ ] `cargo check --workspace` clean; no new warnings.
- [ ] `cargo test --workspace --no-run` compiles.
- [ ] Native `client` and `editor_client` start and pass the relevant smoke.
- [ ] WASM crates compile for `wasm32-unknown-unknown`.
- [ ] Control-API checks pass (see `klep2tron-control`).
- [ ] Docs updated, versions in `README.md`/`PROJECT_MAP.md` match.
