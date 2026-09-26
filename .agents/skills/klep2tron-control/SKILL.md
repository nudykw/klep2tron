---
name: klep2tron-control
description: Control and inspect a running Klep2tron client or editor over its local HTTP control API — take screenshots, read game/map state, and send keyboard/mouse input. Use when debugging or testing Klep2tron's rendering, UI, menus, map editor, or gameplay.
---

# Klep2tron control API

The running `client` and `editor_client` binaries expose **KTRL** (Klep2tron
Control Server) — a local HTTP API on `http://127.0.0.1:15703` (`ktrls/1`), when
the control server is enabled.

Enablement:
- **debug builds** — enabled by default.
- **release builds** — disabled unless `settings.json` contains
  `"control": { "enabled": true, "port": 15703 }`.
- Override with `KLEP_CONTROL=0|1` and `KLEP_CONTROL_PORT=<port>`.
- Optional auth: set `"control": { "token": "..." }` or
  `KLEP_CONTROL_TOKEN=...`; every request must then send
  `Authorization: Bearer <token>` (`401` otherwise).

The server binds to `127.0.0.1` only. Start the app first
(`cargo run -p editor_client` or `cargo run -p client`).

> The bash examples below omit the auth header. Add
> `-H 'Authorization: Bearer <token>'` when a token is configured.

## ktrl CLI

The `ktrl` crate wraps the API in a `clap` binary (uses `ureq`), so you do not
have to hand-roll `curl`. Build once and use it for the common calls:

```bash
cargo build -p ktrl
K=./target/debug/ktrl
$K --url http://127.0.0.1:15703 --token <tok> state
$K version
$K action StartEditor && $K step --frames 40
$K set-tile 3 4 --h 2 --type WedgeN
$K batch --json '[{"op":"action","name":"Undo"},{"op":"state"}]'
$K spawn --mesh cube --name probe --pos 5,1,5 --color 0,1,0
$K move <id> --pos 1,0,0 --relative
$K despawn <id>
$K tree --depth 3
$K entity 813 && $K mesh 813 && $K material 813
$K ui --label Wedge
$K logs --tail 50                     # recent log lines
$K logs --since 1234 --level warn     # incremental / filtered
$K events                             # live log/state/panic stream
$K events --kind state                # only state changes
$K shot -o /tmp/shot.png              # primary window
$K shot --view camera:529 -o /tmp/rtt.png
$K scenario /tmp/scenario.json --verbose
$K record --out /tmp/frames --frames 120 --every 2
$K text 'Actor Name'
$K gamepad --button South
$K gamepad --axis LeftStickX --value 0.5
$K step --frames 10 && $K pause --off
```

`--url`/`--token` default to `KLEP_CONTROL_URL` / `KLEP_CONTROL_TOKEN`.
The library client (`ktrl::Client`) exposes the same calls programmatically.

## Endpoints

### `GET /state`
Returns JSON:

```json
{
  "game_state": "InGame",
  "editor_active": true,
  "frame": 4211,
  "paused": false,
  "fps": 60.0,
  "selection": { "x": 15, "z": 0 },
  "tool": "Cube",
  "undo": 3,
  "redo": 0,
  "map": {
    "current_room": 0,
    "rooms": 1,
    "size": [16, 16],
    "cells": [[{"h":1,"tt":"WedgeE"}, ...], ...]   // cells[x][z]
  },
  "entities": [{"entity": 42, "name": "Player", "pos": [15.0, 0.25, 0.0]}, ...],
  "entities_truncated": false
}
```

- `frame` increments every rendered frame, even while paused — use it as a
  readiness barrier (act, then poll until `frame` advanced).
- `tool` / `undo` / `redo` are supplied by the editor binary; they are absent
  in the plain game client.

### `GET /screenshot`
Returns a PNG of the current frame (`image/png`). Save and open it:

```bash
curl -s --max-time 8 -o /tmp/shot.png http://127.0.0.1:15703/screenshot
# camera / render-target addressed (see /scene_tree for camera entity ids):
curl -s -o /tmp/rtt.png 'http://127.0.0.1:15703/screenshot?view=camera:529'
```

`view` is `primary` (default), `camera:<entity>`, `rtt:<entity>`, or a bare
entity index. Image-render-target cameras (the editor's RTT tile previews) are
captured **offscreen**, so they work even when the window is occluded — use this
to debug the preview thumbnails.

> Debug builds run with continuous rendering (`WinitSettings`), so the window
> does not need focus for a fresh frame. A fully occluded window can still
> return a stale frame on macOS when capturing the primary window.

### `GET /version`
Server identity, useful because `client` and `editor_client` may lag each other:

```json
{"name":"KTRL","server":"ktrls","api":1,"binary":"editor_client","port":15703,"frame":42}
```

### `POST /pause` / `POST /step` — deterministic frames

`/pause` freezes virtual time (animations, camera panning) while rendering and
input keep running. `/step` advances exactly N frames and **blocks until they
have rendered**, returning the final frame number — use it instead of `sleep`.

```bash
curl -s -X POST http://127.0.0.1:15703/pause -d '{"on":true}'
curl -s -X POST http://127.0.0.1:15703/step  -d '{"frames":30}'   # {"ok":true,"frame":..,"paused":true}
curl -s -X POST http://127.0.0.1:15703/pause -d '{"on":false}'
```

`frames` is clamped to 120 (the HTTP request would time out above that).

### `POST /action` — direct actions (no clicking)

Runs an action straight in the ECS. Generic actions work in both binaries;
editor actions require the editor to be active.

```bash
curl -s -X POST http://127.0.0.1:15703/action -d '{"action":"StartEditor"}'
curl -s -X POST http://127.0.0.1:15703/action -d '{"action":"SetTile","x":3,"z":4,"h":2,"tt":"WedgeN"}'
curl -s -X POST http://127.0.0.1:15703/action -d '{"action":"Undo"}'
```

Actions: `StartGame`, `StartEditor`, `QuitToMenu`, `Exit`,
`SetSelection {x,z}`, `SetTile {x,z,h,tt}`, `SetTileType {tt}`, `Undo`, `Redo`,
`NextRoom`, `PrevRoom`, `AddRoom`, `ClearRoom`, `SaveMap`, `LoadMap`,
`SpawnEntity`, `DespawnEntity`, `SetTransform`.
`tt` accepts `Cube`, `Wedge N`/`WedgeN`/`n`, … and `Empty`. Unknown actions get
`400`; the effect lands on the next frame (barrier with `/step`).

**Mutations (fixtures without clicking):**

```bash
# spawn a plain entity (not MapEntity, so it survives map rebuilds):
curl -s -X POST http://127.0.0.1:15703/action \
  -d '{"action":"SpawnEntity","mesh":"cube","name":"probe","translation":[5,1,5],"color":[0,1,0,1]}'
# the new id appears as `last_spawned` in GET /state
curl -s -X POST http://127.0.0.1:15703/action \
  -d '{"action":"SetTransform","id":1009,"translation":[1,0,0],"relative":true}'
curl -s -X POST http://127.0.0.1:15703/action -d '{"action":"DespawnEntity","id":1009}'
```

`SetTransform` takes `translation`, `scale`, `rotation` (quat `[x,y,z,w]`),
`rotation_euler_deg` (`[x,y,z]`) and `relative` (delta for translation/scale).
`SpawnEntity` takes `mesh` (`cube`|`wedge`), `name`, `translation`, `scale`,
`rotation_euler_deg`, `material` (`highlight`) or `color`.

### `POST /batch` — several steps, one request

Runs synchronous steps in order **within a single frame** and returns one
`{status, body}` per step:

```bash
curl -s -X POST http://127.0.0.1:15703/batch -d '{"steps":[
  {"op":"action","name":"SetTile","set":{"x":2,"z":2,"h":1,"tt":"Cube"}},
  {"op":"key","key":"Enter"},
  {"op":"ui_click","label":"Cube"},
  {"op":"state"}
]}'
# {"results":[{"status":200,"body":"{…}"}, …]}
```

Ops: `action` (`name` + optional `set`), `pause`, `key`, `text`, `mouse`,
`ui_click`, `ui_hover`, `unhover`, `state`, `version`, `ui_query`, `logs`,
`tree`/`scene_tree`, `entity`, `mesh`, `material`. `step`/`frames` and
`shot`/`screenshot` are **rejected** (`400`) — `/batch` cannot block or go async.

> Same-frame ordering: a `state` step does not see effects that run in other
> schedules (button actions run in `Update`, map edits land next frame). When
> order across frames matters, use `ktrl scenario` with `/step` barriers.

### `GET /ui_query` — list widgets

Lists UI nodes with their label and layout, so you can click by coordinates or
find the exact label for `/ui_click`:

```json
{"widgets":[{"entity":12,"label":"Cube","interaction":"None",
             "size":[70.0,70.0],"center":[600.0,42.5],
             "min":[565.0,7.5],"max":[635.0,77.5]}]}
```

```bash
curl -s 'http://127.0.0.1:15703/ui_query'                 # all widgets
curl -s 'http://127.0.0.1:15703/ui_query?label=Wedge%20S' # substring filter
```

Coordinates are logical pixels (window space, top-left origin).

### `GET /scene_tree` — entity hierarchy

```bash
curl -s 'http://127.0.0.1:15703/scene_tree?depth=3'        # whole scene
curl -s 'http://127.0.0.1:15703/scene_tree?root=42&depth=2' # subtree
```

```json
{"roots":[{"entity":42,"name":"Player","kind":"mesh","children":[…] }],
 "depth":3,"entity_count":972,"truncated":false}
```

`kind` is one of `camera`, `mesh`, `ui_node`, `ui_text`, `light`, `entity`.
Nodes without a `Name` have `"name":null`; the editor does not name its tiles.
Node budget is 1024; `children_truncated` marks cut subtrees.

### `GET /entity/{id}` — one entity

```bash
curl -s http://127.0.0.1:15703/entity/813
# or: /entity?id=813
```

Returns `name`, `parent`, `kind`, the `components` list, `transform`
(translation/rotation/scale), `mesh`/`material` asset ids, `text`, and
`render_target` for cameras.

### `GET /mesh/{id}` — mesh of an entity

```json
{"entity":813,"handle":"AssetId<...>{ index: 7, generation: 0}",
 "vertex_count":18,"index_count":24,"topology":"TriangleList",
 "attributes":[{"name":"Vertex_Position","count":18}, …]}
```

`attributes_available:false` means the CPU-side mesh data was released after
extraction to the render world (vertex counts are then `null`).

### `GET /material/{id}` — material of an entity

Reports `base_color`, `emissive`, `perceptual_roughness`, `metallic`,
`reflectance`, `unlit`, `double_sided`, `cull_mode`, `alpha_mode` and the
`base_color_texture` / `emissive_texture` asset ids of the entity's
`StandardMaterial`.

### `GET /logs` — recent log lines

The binaries buffer the last 1000 log events (via a `tracing` layer), so you
can read warnings/errors without scraping stdout:

```json
{"entries":[{"seq":437,"ts_ms":1758880000000,"level":"WARN",
              "target":"bevy_render","message":"…"}],
 "last_seq":437}
```

```bash
curl -s 'http://127.0.0.1:15703/logs?tail=50'
curl -s 'http://127.0.0.1:15703/logs?since=437'          # only new entries
curl -s 'http://127.0.0.1:15703/logs?level=warn'         # warn+ only
```

With `since`, `tail` is ignored and every newer entry is returned (poll with the
previous `last_seq`).

### `GET /events` — live event stream (SSE)

`text/event-stream` with `log`, `state` and `panic` events. `data` is JSON
(logs: a `/logs` entry; state: `{game_state,editor_active,room,rooms,frame}`;
panic: `{message}`). The stream keeps the connection open and sends keep-alive
comments; stop with Ctrl-C.

```bash
curl -sN http://127.0.0.1:15703/events
# event: state
# data: {"game_state":"InGame","editor_active":true,"frame":234,...}
```

Use this instead of polling `/state`/`/logs` for long-running observation.

### `POST /key`
`{"key":"ArrowUp","action":"tap"|"press"|"release"}` (default `tap`).

```bash
curl -s -X POST http://127.0.0.1:15703/key -d '{"key":"Enter"}'
```

Recognised names: arrows, `Enter`, `Space`, `Escape`, `Tab`, `Backspace`,
`Delete`, `ShiftLeft`, `ControlLeft`, `AltLeft`, `F1..F5`, `KeyA..KeyZ`,
`Digit0..Digit9`, or a single letter/digit.

### `POST /text` — type into a focused field

Synthesizes one `KeyboardInput` per character, so focused text fields (e.g. the
actor editor's name field) receive it:

```bash
curl -s -X POST http://127.0.0.1:15703/text -d '{"text":"Actor Name"}'
# {"ok":true,"chars":10}
```

`\n`, `\t` and `\u{8}` map to Enter, Tab and Backspace (Enter/Escape also blur
a field). Focus a field first (via `/ui_click` or `/mouse`).

### `POST /gamepad` — inject gamepad input

Spawns a virtual gamepad (visible in `/scene_tree` as `KTRL virtual gamepad`)
and injects the same `RawGamepadEvent`s `bevy_gilrs` emits. Bevy processes them
on the **next frame**, so follow with `/step`.

```bash
curl -s -X POST http://127.0.0.1:15703/gamepad -d '{"button":"South"}'            # tap (auto-release)
curl -s -X POST http://127.0.0.1:15703/gamepad -d '{"button":"DPadDown","action":"press"}'
curl -s -X POST http://127.0.0.1:15703/gamepad -d '{"button":"DPadDown","action":"release"}'
curl -s -X POST http://127.0.0.1:15703/gamepad -d '{"axis":"LeftStickX","value":0.5}'
```

Buttons: `South`/`A`, `East`/`B`, `North`/`Y`, `West`/`X`, `DPadUp/Down/Left/Right`,
`Start`, `Select`/`Back`, `LeftTrigger`/`LT`, `RightTrigger`/`RT`, thumbs.
`action` is `tap` (default; queues a release after two frames), `press` or
`release`. Buttons stay `pressed` until released — use `tap` for discrete
navigation. Axes persist until overwritten.

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

**Walk the main menu** (keyboard driven). Use `/step` as the barrier instead
of fixed sleeps, so it works at any frame rate:
```bash
for k in ArrowDown ArrowDown Enter; do
  curl -s -X POST http://127.0.0.1:15703/key -d "{\"key\":\"$k\"}"
  curl -s -X POST http://127.0.0.1:15703/step -d '{"frames":10}'
done
curl -s http://127.0.0.1:15703/state
```

**Place a tile in the map editor** — no clicking needed:
```bash
curl -s -X POST http://127.0.0.1:15703/action -d '{"action":"SetTile","x":4,"z":7,"h":3,"tt":"WedgeW"}'
curl -s -X POST http://127.0.0.1:15703/step  -d '{"frames":3}'
```
Mouse/keyboard still work too (`Q`/`A` change height, arrows move selection).

## Checking the editor preview thumbnails

`scripts/preview_check.py` reports which of the five tile-type thumbnails in
the editor's top panel actually rendered:

```bash
python3 -m venv /tmp/imgvenv && /tmp/imgvenv/bin/pip install pillow
curl -s -o /tmp/shot.png http://127.0.0.1:15703/screenshot
/tmp/imgvenv/bin/python scripts/preview_check.py /tmp/shot.png
# e.g. [5-555]  -> Cube, WedgeW, WedgeN, WedgeE visible; WedgeS blank
```

`scripts/editor_smoke.sh` automates the full regression: enter the editor, run
slow and fast hover series over all five buttons, and fail if any shot is not
`[55555]`:

```bash
scripts/editor_smoke.sh                      # control API on 127.0.0.1:15703
scripts/editor_smoke.sh http://127.0.0.1:15703
```

`scripts/api_smoke.sh` is the full end-to-end API check (version/state/logs,
pause/step, the editor round-trip, introspection, tiles, mutations, key/text,
batch, primary + offscreen screenshots and SSE). Run it against any running
debug build:

```bash
scripts/api_smoke.sh
KLEP_CONTROL_TOKEN=<tok> scripts/api_smoke.sh
```

See `references/editor-verification.md` for the full set of recipes, including
the orbit-camera (RTT rotation) check and why `/ui_hover` alone may not
reproduce archetype-order bugs.

## Notes

- Screenshots are captured by the engine itself, so no OS screen-recording
  permission is needed — but the window must not be occluded.
- Prefer `/step` over `sleep`: it blocks until the requested frames rendered and
  returns the frame number. Only mouse/keys need a real time gap inside the app.
- The API is intended for local debugging only.
