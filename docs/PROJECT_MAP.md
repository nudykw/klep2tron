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
  - `src/actor_editor/`: Modular system for creating and editing NPC actors.
    - `ui/layout/`: Global layout, cameras, and lighting.
    - `ui/inspector/`: Detailed part and socket inspection.
    - `systems/`: Input, gizmos, slicing, and normalization logic.
  - `src/input.rs`: Global shared controls (fullscreen).
  - `src/history.rs`: Undo/Redo stack management.
- `crates/client_web`: WASM wrapper for the game client.
- `crates/editor_client`: Native map editor.
  - `src/camera.rs`: Orbit controls and RTT sync.
  - `src/ui/`: Editor-specific buttons and tooltips.
  - `src/logic.rs`: Selection, mouse mapping, and history.
- `crates/editor_client_web`: WASM wrapper for the map editor.
- `crates/server`: Server-side logic (Docker/PostgreSQL).
- `crates/shared`: Shared data structures between client and server.
- `crates/admin_web`: Web interface for administration.

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
