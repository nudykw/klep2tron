---
name: klep2tron-control
description: Control and inspect a running Klep2tron client or editor over its local HTTP control API — take screenshots, read game/map state, and send keyboard/mouse input. Use when debugging or testing Klep2tron's rendering, UI, menus, map editor, or gameplay.
---

# Klep2tron control API

The running `client` and `editor_client` binaries expose a local HTTP API on
`http://127.0.0.1:15703` when the control server is enabled.

Enablement:
- **debug builds** — enabled by default.
- **release builds** — disabled unless `settings.json` contains
  `"control": { "enabled": true, "port": 15703 }`.
- Override with `KLEP_CONTROL=0|1` and `KLEP_CONTROL_PORT=<port>`.

The server binds to `127.0.0.1` only. Start the app first
(`cargo run -p editor_client` or `cargo run -p client`).

## Endpoints

### `GET /state`
Returns JSON:

```json
{
  "game_state": "InGame",
  "editor_active": true,
  "selection": { "x": 15, "z": 0 },
  "map": {
    "current_room": 0,
    "rooms": 1,
    "cells": [[{"h":1,"tt":"WedgeE"}, ...], ...]   // cells[x][z]
  },
  "entities": [{"entity": 42, "pos": [15.0, 0.25, 0.0]}, ...]
}
```

### `GET /screenshot`
Returns a PNG of the current frame (`image/png`). Save and open it:

```bash
curl -s --max-time 8 -o /tmp/shot.png http://127.0.0.1:15703/screenshot
```

> Debug builds run with continuous rendering (`WinitSettings`), so the window
> does not need focus for a fresh frame. A fully occluded window can still
> return a stale frame on macOS — keep the app window visible when possible.

### `POST /key`
`{"key":"ArrowUp","action":"tap"|"press"|"release"}` (default `tap`).

```bash
curl -s -X POST http://127.0.0.1:15703/key -d '{"key":"Enter"}'
```

Recognised names: arrows, `Enter`, `Space`, `Escape`, `Tab`, `Backspace`,
`Delete`, `ShiftLeft`, `ControlLeft`, `AltLeft`, `F1..F5`, `KeyA..KeyZ`,
`Digit0..Digit9`, or a single letter/digit.

### `POST /mouse`
```bash
curl -s -X POST http://127.0.0.1:15703/mouse -d '{"x":1200,"y":800}'          # move
curl -s -X POST http://127.0.0.1:15703/mouse -d '{"button":"left","action":"click"}'
```

> Injected mouse movement does not drive Bevy's UI picking (that follows real
> winit pointer events), so prefer `/ui_click` for UI buttons.

### `POST /ui_click` / `POST /ui_hover`
Set a UI button's `Interaction` directly, matched by its `Name` or `Text`.
Useful for UI that mouse injection cannot reach.

```bash
# one-frame press (triggers the button's action)
curl -s -X POST http://127.0.0.1:15703/ui_click -d '{"label":"LEVEL EDITOR","action":"click"}'
# keep it hovered (re-applied every frame until cleared)
curl -s -X POST http://127.0.0.1:15703/ui_hover -d '{"label":"Wedge S","action":"hover"}'
curl -s -X POST http://127.0.0.1:15703/ui_hover -d '{"action":"unhover"}'
```

## Typical workflows

**Capture what the app is doing:**
```bash
curl -s http://127.0.0.1:15703/state | python3 -m json.tool | head -40
curl -s -o /tmp/shot.png http://127.0.0.1:15703/screenshot
```

**Walk the main menu** (keyboard driven):
```bash
for k in Down Down Enter; do
  curl -s -X POST http://127.0.0.1:15703/key -d "{\"key\":\"$k\"}"; sleep 0.4
done
curl -s http://127.0.0.1:15703/state
```

**Place a tile in the map editor:** focus the window, then move/click with
`/mouse` and select the tile type in the top panel, or use keyboard shortcuts
(`Q`/`A` change height, arrow keys move the selection).

## Checking the editor preview thumbnails

`scripts/preview_check.py` reports which of the five tile-type thumbnails in
the editor's top panel actually rendered:

```bash
python3 -m venv /tmp/imgvenv && /tmp/imgvenv/bin/pip install pillow
curl -s -o /tmp/shot.png http://127.0.0.1:15703/screenshot
/tmp/imgvenv/bin/python scripts/preview_check.py /tmp/shot.png
# e.g. [5-555]  -> Cube, WedgeW, WedgeN, WedgeE visible; WedgeS blank
```

## Notes

- Screenshots are captured by the engine itself, so no OS screen-recording
  permission is needed — but the window must not be occluded.
- Some endpoints act on the next frame; wait ~100–300 ms between calls.
- The API is intended for local debugging only.
