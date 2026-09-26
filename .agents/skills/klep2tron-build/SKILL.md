---
name: klep2tron-build
description: Build, run, and package Klep2tron across native, WASM/Trunk, and server/Docker targets. Use for build or run failures, platform setup, Cargo feature flags, asset-loading paths, or producing web and server artifacts.
---

# Building and running Klep2tron

Cargo workspace, Rust **edition 2021**. Native/runtime checks run on macOS
(Apple Silicon); Linux and WASM are supported.

## Native

```bash
cargo check --workspace
cargo build --workspace
cargo test --workspace --no-run          # tests are sparse; this only compiles them

cargo run -p editor_client               # map editor
cargo run -p client                      # game client
cargo run -p server                      # authoritative server
```

`cargo run -p <crate>` sets `CARGO_MANIFEST_DIR`; running the binary directly
does not. If you launch `./target/debug/editor_client` yourself, set the asset
root or Bevy looks in `target/debug/assets`:

```bash
BEVY_ASSET_ROOT=$PWD ./target/debug/editor_client
```

## Web (WASM) via Trunk

Install once: `cargo install --locked trunk` (and `rustup target add
wasm32-unknown-unknown`).

```bash
cd crates/client_web        && trunk serve   # http://127.0.0.1:8081
cd crates/editor_client_web && trunk serve   # http://127.0.0.1:8082
```

Per-crate config lives in `crates/*_web/Trunk.toml`; assets in
`crates/*_web/assets`. `actor_editor` is **native-only** and must not be pulled
into web crates.

WASM needs JS shims: web crates use `getrandom` with the `js`/`wasm_js`
features and `uuid` with `js`. If a wasm build complains about randomness, check
those features rather than the game code.

## Feature flags

| Context | `bevy` features |
|---|---|
| Native (`client`, `editor_client`, `client_core`, `actor_editor`) | `["default", "wayland"]` |
| Web (`client_web`, `editor_client_web`) | `["webgl2"]` |

Adding a native-only feature to a web crate, or vice versa, is a common mistake.

## Platform setup

Full instructions (Debian/Ubuntu, Arch, Fedora, macOS/Homebrew, Windows) are in
`README.md`. Linux needs Bevy's system libs: `libx11`, `libxcursor`, `libxi`,
`libxrandr`, `libxinerama`, `libwayland`, `libxkbcommon`, `libasound2`,
`libudev`/`eudev`, `pkg-config`/`pkgconf`. macOS needs Xcode Command Line Tools.

## Server and Docker

- `cargo run -p server` needs PostgreSQL; `DATABASE_URL` example in
  `docker-compose.yml` (`postgres://klepto:password@db:5432/kleptotron`).
- `docker compose up --build` builds `server` via `Dockerfile` (release) and
  starts Postgres. The server exposes `5000/udp`.
- `admin_web` is an Axum admin panel; run it with `cargo run -p admin_web`.

## Common failures

| Symptom | Cause / fix |
|---|---|
| `Path not found: .../target/debug/assets/...` | Running the binary directly without `BEVY_ASSET_ROOT=$PWD` |
| `no method named ...` after adding a helper | `let`-chains need edition 2024; this workspace is edition 2021 — use nested `if let` |
| `ld: __eh_frame section too large ...` (macOS) | Benign linker warning, ignore |
| Two versions of `wgpu` / `bevy_*` in `cargo tree` | Keep ecosystem crates on the matching Bevy minor; `wgpu` is driven by Bevy |
| WASM link error about `getrandom` | Add the `js`/`wasm_js` feature on the right `getrandom` version |
| `actor_editor` fails to compile for wasm | It is native-only by design; do not add it to web crates |
| Pointer picking does nothing | Injected mouse input does not drive `bevy_picking`; use `/ui_hover` / `/ui_click` (see `klep2tron-control`) |

## Artifacts and ignored files

`/target`, `**/dist/`, `*.wasm`, `*.js`, `map.json`, `settings.json`,
`perf_metrics.json` are git-ignored. The editor auto-saves the map to
`map.json` (git-ignored) — do not commit it.

## Verification

- [ ] `cargo check --workspace` (or affected crates).
- [ ] `cargo check -p client_web --target wasm32-unknown-unknown` and
      `editor_client_web` when UI/asset code changed.
- [ ] Native binaries start and pass the relevant control-API smoke
      (`klep2tron-control`).
- [ ] Release build sanity when touching the profile: `cargo build --release -p <crate>`.
