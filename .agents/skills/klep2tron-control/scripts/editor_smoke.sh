#!/usr/bin/env bash
#
# Smoke-test the map editor's tile-type preview thumbnails over the control API.
#
# Enters the level editor, runs slow and fast hover series over the five
# tile-type buttons, and fails if any thumbnail is missing ([5-555] etc.).
#
# Requires: a running debug build (client or editor_client), curl, and Pillow.
# Keep the app window visible: an occluded macOS window can return stale frames.
#
# Usage:
#   scripts/editor_smoke.sh [BASE_URL]
# Env:
#   PREVIEW_PY   python with Pillow (default: /tmp/imgvenv/bin/python)
#
# See references/editor-verification.md for the underlying recipes.

set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECK="$HERE/preview_check.py"
S="${1:-http://127.0.0.1:15703}"
PY="${PREVIEW_PY:-/tmp/imgvenv/bin/python}"
TMP="${TMPDIR:-/tmp}"

if [ ! -x "$PY" ]; then
  echo "Pillow venv not found at $PY. Create it with:" >&2
  echo "  python3 -m venv /tmp/imgvenv && /tmp/imgvenv/bin/pip install pillow" >&2
  exit 2
fi

curl -s --max-time 5 "$S/state" >/dev/null 2>&1 || {
  echo "control API not reachable at $S (is the app running in a debug build?)" >&2
  exit 2
}

snap()  { curl -s --max-time 8 -o "$1" "$S/screenshot"; }
hover() { curl -s -X POST "$S/ui_hover" -d "$1" >/dev/null; }
key()   { curl -s -X POST "$S/key"      -d "$1" >/dev/null; }
bits()  { "$PY" "$CHECK" "$1" | awk '{print $2}'; }

# Enter the level editor when we are still in the menu.
if curl -s --max-time 5 "$S/state" | grep -q '"game_state":"Menu"'; then
  key '{"key":"ArrowDown"}'; sleep 0.3
  key '{"key":"Enter"}';     sleep 0.5
fi

# Offscreen render targets need a moment to warm up.
sleep 2.5

shots=()

# Capture, retrying while the frame is completely empty (warmup or a stale /
# occluded frame). A real regression leaves a *partial* result such as
# [5-555], which is not retried away.
shot() {
  local file="$1" result i
  for i in 1 2 3 4 5 6; do
    snap "$file"
    result="$(bits "$file")"
    if [ "$result" != "[-----]" ]; then
      echo "$(basename "$file") $result"
      shots+=("$file")
      return 0
    fi
    sleep 0.5
  done
  echo "$(basename "$file") $result  (no content after retries)" >&2
  shots+=("$file")
}

labels=("Cube" "Wedge S" "Wedge W" "Wedge N" "Wedge E")

echo "== baseline =="
shot "$TMP/klep_smoke_base.png"

echo "== slow hover (hold 1.0 s) =="
for i in 0 1 2 3 4; do
  hover "{\"label\":\"${labels[$i]}\"}"
  sleep 1.0
  shot "$TMP/klep_smoke_slow_$i.png"
  hover '{"action":"unhover"}'
  sleep 0.3
done

echo "== fast hover =="
for i in 0 1 2 3 4; do
  hover "{\"label\":\"${labels[$i]}\"}"
  hover '{"action":"unhover"}'
  sleep 0.4
  shot "$TMP/klep_smoke_fast_$i.png"
done

fail=0
for file in "${shots[@]}"; do
  result="$(bits "$file")"
  if [ "$result" != "[55555]" ]; then
    echo "FAIL: $file -> $result" >&2
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  echo "editor preview smoke FAILED" >&2
  exit 1
fi

echo "editor preview smoke OK"
