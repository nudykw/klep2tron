#!/usr/bin/env bash
#
# End-to-end smoke test for the KTRL control API.
#
# Exercises every endpoint against a *running* debug build and fails on the
# first broken assertion. Deterministic frame stepping (`/step`) is used instead
# of sleeps, so it works at any frame rate.
#
# Requires: a running `client`/`editor_client` (debug), curl, python3.
# Usage:  scripts/api_smoke.sh [BASE_URL]
# Env:    KLEP_CONTROL_TOKEN   bearer token, when the server requires one.
#
# See references/editor-verification.md and SKILL.md.

set -u

S="${1:-http://127.0.0.1:15703}"
TOKEN="${KLEP_CONTROL_TOKEN:-}"
TMP="${TMPDIR:-/tmp}"
PY=python3
fails=0

# curl with the optional bearer header (bash 3.2-safe: no empty arrays under set -u).
curl_auth() {
  if [ -n "$TOKEN" ]; then curl -H "Authorization: Bearer $TOKEN" "$@"; else curl "$@"; fi
}

ok()  { printf 'ok   %s\n' "$1"; }
bad() { printf 'FAIL %s\n' "$1" >&2; fails=$((fails + 1)); }
get()  { curl_auth -s "$S$1"; }
post() { curl_auth -s -X POST "$S$1" -d "$2"; }
# Check a JSON snippet: `echo "$json" | jqcheck '<expr on d>'`.
jqcheck() { "$PY" -c "import json,sys;d=json.load(sys.stdin);print('yes' if ($1) else 'no')" 2>/dev/null; }
check() { [ "$1" = yes ] && ok "$2" || bad "$2"; }

curl_auth -s -o /dev/null --max-time 5 "$S/version" || {
  echo "KTRL not reachable at $S (run a debug build first)" >&2
  exit 2
}

# --- version / state / logs -------------------------------------------------

v="$(get /version)"
check "$(echo "$v" | jqcheck 'd["name"]=="KTRL" and d["api"]==1')" "GET /version"

st="$(get /state)"
check "$(echo "$st" | jqcheck '"frame" in d and "game_state" in d')" "GET /state"

lg="$(get '/logs?tail=1')"
check "$(echo "$lg" | jqcheck '"entries" in d and "last_seq" in d')" "GET /logs"

# --- pause / step -----------------------------------------------------------

check "$(post /pause '{"on":true}' | jqcheck 'd["paused"]==True')" "POST /pause on"
check "$(post /step '{"frames":3}' | jqcheck '"frame" in d')" "POST /step"
check "$(post /pause '{"on":false}' | jqcheck 'd["paused"]==False')" "POST /pause off"

# --- enter the editor -------------------------------------------------------

state_of() { get /state | "$PY" -c 'import json,sys;print(json.load(sys.stdin)["game_state"])'; }
if [ "$(state_of)" = "Menu" ]; then
  post /action '{"action":"StartEditor"}' >/dev/null
fi
for _ in $(seq 1 20); do
  post /step '{"frames":60}' >/dev/null
  [ "$(state_of)" = "InGame" ] && break
done
check "$(get /state | jqcheck 'd["game_state"]=="InGame" and d.get("editor_active")==True')" "action StartEditor -> InGame"

# --- UI widgets -------------------------------------------------------------

uq="$(get '/ui_query?label=Wedge')"
check "$(echo "$uq" | jqcheck 'len(d["widgets"])>=1')" "GET /ui_query"

# --- scene introspection ----------------------------------------------------

tree="$(get '/scene_tree?depth=1')"
check "$(echo "$tree" | jqcheck 'd["entity_count"]>0')" "GET /scene_tree"

mesh_id="$(echo "$tree" | "$PY" -c 'import json,sys;d=json.load(sys.stdin);print(next((r["entity"] for r in d["roots"] if r["kind"]=="mesh"), ""))')"
if [ -n "$mesh_id" ]; then
  check "$(get "/entity/$mesh_id" | jqcheck 'd["kind"]=="mesh" and "mesh" in d')" "GET /entity/{id}"
  check "$(get "/mesh/$mesh_id" | jqcheck '"topology" in d and "vertex_count" in d')" "GET /mesh/{id}"
  check "$(get "/material/$mesh_id" | jqcheck '"base_color" in d and "alpha_mode" in d')" "GET /material/{id}"
else
  bad "no mesh entity found in scene_tree"
fi

# --- tiles / selection ------------------------------------------------------

post /action '{"action":"SetTile","x":5,"z":6,"h":2,"tt":"WedgeN"}' >/dev/null
post /step '{"frames":3}' >/dev/null
check "$(get /state | jqcheck 'd["map"]["cells"][5][6]["h"]==2 and d["map"]["cells"][5][6]["tt"]=="WedgeN"')" "action SetTile"
check "$(post /action '{"action":"SetSelection","x":5,"z":6}' | jqcheck 'd["ok"]==True')" "action SetSelection"

# --- mutations --------------------------------------------------------------

post /action '{"action":"SpawnEntity","mesh":"cube","name":"api_probe","translation":[4,1,4]}' >/dev/null
post /step '{"frames":3}' >/dev/null
spawned="$(get /state | "$PY" -c 'import json,sys;d=json.load(sys.stdin);print(d.get("last_spawned",""))')"
if [ -n "$spawned" ]; then
  ok "action SpawnEntity (id $spawned)"
  body="{\"action\":\"SetTransform\",\"id\":$spawned,\"translation\":[1,0,0],\"relative\":true}"
  check "$(post /action "$body" | jqcheck 'd["ok"]==True')" "action SetTransform"
  post /step '{"frames":3}' >/dev/null
  check "$(get /state | "$PY" -c "import json,sys;d=json.load(sys.stdin);e=[x for x in d['entities'] if x['entity']==$spawned];print('yes' if e and abs(e[0]['pos'][0]-5.0)<0.01 else 'no')" )" "SetTransform moved entity"
  body="{\"action\":\"DespawnEntity\",\"id\":$spawned}"
  check "$(post /action "$body" | jqcheck 'd["ok"]==True')" "action DespawnEntity"
else
  bad "SpawnEntity did not report last_spawned"
fi

# --- key / text -------------------------------------------------------------

check "$(post /key '{"key":"ArrowUp"}' | jqcheck 'd["ok"]==True')" "POST /key"
check "$(post /text '{"text":"hi"}' | jqcheck 'd["chars"]==2')" "POST /text"

# --- batch ------------------------------------------------------------------

batch="$(post /batch '{"steps":[{"op":"action","name":"Undo"},{"op":"state"},{"op":"step","frames":5}]}')"
check "$(echo "$batch" | jqcheck 'len(d["results"])==3 and d["results"][2]["status"]==400')" "POST /batch (step rejected)"

# --- screenshots ------------------------------------------------------------

curl_auth -s -o "$TMP/klep_api_primary.png" "$S/screenshot"
if [ -s "$TMP/klep_api_primary.png" ] && [ "$(head -c4 "$TMP/klep_api_primary.png" | od -An -tx1 | tr -d ' \n')" = "89504e47" ]; then
  ok "GET /screenshot (primary)"
else
  bad "GET /screenshot (primary) is not a PNG"
fi

rtt_id=""
# Find an image-render-target camera by probing the cameras from the tree.
cameras="$(echo "$tree" | "$PY" -c 'import json,sys;d=json.load(sys.stdin);print(" ".join(str(r["entity"]) for r in d["roots"] if r["kind"]=="camera"))')"
for cam in $cameras; do
  kind="$(get "/entity/$cam" | "$PY" -c 'import json,sys;d=json.load(sys.stdin);rt=d.get("render_target") or {};print(rt.get("kind",""))' 2>/dev/null)"
  if [ "$kind" = "image" ]; then rtt_id="$cam"; break; fi
done
if [ -n "$rtt_id" ]; then
  curl_auth -s -o "$TMP/klep_api_rtt.png" "$S/screenshot?view=camera:$rtt_id"
  if [ -s "$TMP/klep_api_rtt.png" ] && [ "$(head -c4 "$TMP/klep_api_rtt.png" | od -An -tx1 | tr -d ' \n')" = "89504e47" ]; then
    ok "GET /screenshot?view=camera:$rtt_id (offscreen)"
  else
    bad "offscreen screenshot is not a PNG"
  fi
else
  bad "no image-render-target camera found"
fi

# --- SSE --------------------------------------------------------------------

: > "$TMP/klep_api_events.txt"
curl_auth -sN "$S/events" > "$TMP/klep_api_events.txt" 2>/dev/null &
events_pid=$!
sleep 0.5
post /action '{"action":"SetTile","x":1,"z":2,"h":1,"tt":"Cube"}' >/dev/null
post /step '{"frames":3}' >/dev/null
sleep 0.5
kill "$events_pid" 2>/dev/null
if grep -q '^event: ' "$TMP/klep_api_events.txt"; then
  ok "GET /events (SSE)"
else
  bad "GET /events produced no events"
fi

# --- result -----------------------------------------------------------------

if [ "$fails" -ne 0 ]; then
  echo "api smoke FAILED ($fails)" >&2
  exit 1
fi
echo "api smoke OK"
