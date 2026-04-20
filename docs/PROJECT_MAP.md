# 🗺️ Klep2Tron Project Map

This document serves as a technical overview for AI assistance to quickly navigate and understand the codebase.

## 🏗️ Workspace Structure

- `crates/client`: Native game client binary.
- `crates/client_core`: Shared game logic, rendering systems, and asset management (The "Heart").
- `crates/client_web`: WASM wrapper for the game client.
- `crates/editor_client`: Native map editor binary.
- `crates/editor_client_web`: WASM wrapper for the map editor.
- `crates/server`: Server-side logic (Docker/PostgreSQL).
- `crates/shared`: Shared data structures between client and server (e.g., `Project`, `Room`, `Cell`).
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

## 🎨 Rendering Logic (`map_rendering_system`)

Located in `client_core/src/lib.rs`.
- Uses **Partial Updates**: Only tiles in `DirtyTiles` are re-spawned.
- **Full Rebuild**: Triggered by room change or `dirty.full_rebuild`.
- **Change Detection**: Uses `project.is_changed()` to detect external loads.

## 🛠️ Editor Specifics

- **OrbitCamera**: Specialized 3D camera controller for the editor.
- **RTT Previews**: Render-to-Texture previews of tile types in the top panel.
- **Gizmos**: Used for selection highlights and dashed wedge outlines.

## 🚀 Common Commands

- **Run Editor**: `cargo run -p editor_client`
- **Run Web Editor**: `trunk serve --port 8082` (in `crates/editor_client_web`)
- **Run Game**: `cargo run -p client`
- **Check All**: `cargo check --workspace`
