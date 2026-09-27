# Editor UI verification recipes

Recipes for verifying map-editor UI with the control API. The editor's top
panel shows five tile-type thumbnails rendered offscreen into textures
(`RenderTarget::Image`) and displayed through `ImageNode`. The map is
`Cube, Wedge S, Wedge W, Wedge N, Wedge E` from left to right.

## Prerequisites

- The app is running in a debug build (control API enabled by default):
  `BEVY_ASSET_ROOT=$PWD ./target/debug/editor_client` or `cargo run -p editor_client`.
- The window is visible; an occluded macOS window can return stale frames.
- Pillow in a venv:

```bash
python3 -m venv /tmp/imgvenv && /tmp/imgvenv/bin/pip install pillow
PREVIEW_PY=/tmp/imgvenv/bin/python
```

`scripts/editor_smoke.sh` uses `/tmp/imgvenv/bin/python` by default; override
with the `PREVIEW_PY` environment variable.

## Enter the level editor

From the main menu: `ArrowDown` selects LEVEL EDITOR, `Enter` activates it.

```bash
S=http://127.0.0.1:15703
curl -s -X POST $S/key -d '{"key":"ArrowDown"}'
sleep 0.3
curl -s -X POST $S/key -d '{"key":"Enter"}'
sleep 0.5
curl -s $S/state | python3 -c 'import sys,json;d=json.load(sys.stdin);print(d["game_state"], d["editor_active"])'
# InGame True
```

Render targets need a moment after entering: the first few hundred ms can read
`[-----]` while the offscreen cameras and pipelines warm up. That is expected;
a thumbnail that is still blank after ~2 s is a failure.

## Interpret `preview_check.py`

```bash
curl -s --max-time 8 -o /tmp/shot.png $S/screenshot
$PREVIEW_PY scripts/preview_check.py /tmp/shot.png
# [5-555]  -> Cube, WedgeW, WedgeN, WedgeE visible; Wedge S blank
# [55555]  -> all five visible
```

The script samples only the exact x-windows of a **2560x1440** window. If the
window size changes, update `WINDOWS` in `scripts/preview_check.py`.

## Slow and fast hover regression

This guards the bug described in `plans/Preview_RTT_Thumbnails_Bug.md`: a slow
first hover could permanently hide a thumbnail. Slow hover holds
`Interaction::Hovered` for many frames; fast hover only flashes it.

```bash
for label in "Cube" "Wedge S" "Wedge W" "Wedge N" "Wedge E"; do
  # slow: hold 1.5 s, then unhover
  curl -s -X POST $S/ui_hover -d "{\"label\":\"$label\"}"
  sleep 1.5
  curl -s --max-time 8 -o "/tmp/slow_${label// /_}.png" $S/screenshot
  curl -s -X POST $S/ui_hover -d '{"action":"unhover"}'
  sleep 0.4

  # fast: hover + unhover immediately, then capture
  curl -s -X POST $S/ui_hover -d "{\"label\":\"$label\"}"
  curl -s -X POST $S/ui_hover -d '{"action":"unhover"}'
  sleep 0.4
  curl -s --max-time 8 -o "/tmp/fast_${label// /_}.png" $S/screenshot
done
$PREVIEW_PY scripts/preview_check.py /tmp/slow_*.png /tmp/fast_*.png
```

Every line must be `[55555]`. `scripts/editor_smoke.sh` runs this automatically
and exits non-zero on failure.

## Orbit-camera / RTT rotation check

`sync_rtt_cameras_system` keeps the five offscreen cameras orbiting with the
main camera. Do **not** break this. Verify that the thumbnails still render and
that their pixels actually change after an orbit:

```bash
curl -s --max-time 8 -o /tmp/orbit_before.png $S/screenshot
curl -s -X POST $S/key -d '{"key":"ShiftLeft","action":"press"}'
curl -s -X POST $S/key -d '{"key":"ArrowLeft","action":"press"}'
sleep 1.2
curl -s -X POST $S/key -d '{"key":"ArrowLeft","action":"release"}'
curl -s -X POST $S/key -d '{"key":"ShiftLeft","action":"release"}'
sleep 0.6
curl -s --max-time 8 -o /tmp/orbit_after.png $S/screenshot
$PREVIEW_PY scripts/preview_check.py /tmp/orbit_before.png /tmp/orbit_after.png
```

Then confirm the tiles rotated (not just that they are present):

```bash
$PREVIEW_PY - <<'PY'
from PIL import Image, ImageChops
a = Image.open('/tmp/orbit_before.png').convert('RGB')
b = Image.open('/tmp/orbit_after.png').convert('RGB')
d = ImageChops.difference(a.crop((800, 20, 1600, 180)), b.crop((800, 20, 1600, 180)))
h = d.convert('L').histogram()
print("changed pixels:", sum(h) - h[0])
PY
```

A non-zero, clearly large count means the previews followed the orbit.

## Advanced: archetype-move repro

`/ui_hover` only sets the `Interaction` value, so it does **not** always
reproduce bugs that need a component insertion (an archetype move). To verify
an ordering fix deterministically, temporarily add a debug system that inserts
a throwaway marker component on one tile button a couple of seconds after
entering the editor, capture a screenshot, then check the `ComputedStackIndex`
of the button and its `ImageNode`:

- correct: `ImageNode` stack index is exactly `button + 1`;
- buggy: the button is ordered **after** its own image and its opaque
  background covers the thumbnail (`[5555-]`).

Remove the debug system after verifying. The original 30-frame
`PreviewImagePending` workaround was itself such a move, which is why it never
fixed the bug.

## Limitations

- `preview_check.py` windows assume a 2560x1440 window.
- Screenshots come from the engine, so no screen-recording permission is
  needed, but the window must not be fully occluded.
- Give endpoints ~100–300 ms between calls; `/key` and `/ui_*` act on the next
  frame.
