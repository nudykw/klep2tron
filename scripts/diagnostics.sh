#!/usr/bin/env bash
#
# Structured cargo diagnostics for agents and humans.
#
# Runs `cargo check --workspace --message-format=json` (or `cargo clippy` when
# CARGO_CMD=clippy) and prints one line per diagnostic:
#
#     ERROR[E0308] crates/foo/src/lib.rs:12:9: mismatched types
#         ↳ expected `u32`, found `i32`
#
# followed by a one-line summary on stderr. Exit code is non-zero when there
# are errors, so it can gate a task. Unlike raw `cargo check`, the output is
# compact and machine-readable by eye (no rustc rendering, no truncation).
#
# Usage:
#   scripts/diagnostics.sh                     # cargo check --workspace
#   CARGO_CMD=clippy scripts/diagnostics.sh    # cargo clippy --workspace
#   scripts/diagnostics.sh -p editor_client    # forward extra args to cargo
#
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cargo_cmd="${CARGO_CMD:-check}"

# Default to the whole workspace unless the caller already selected packages.
select_scope=1
for arg in "$@"; do
    case "$arg" in
        -p|--package|--workspace|--all|--exclude) select_scope=0 ;;
    esac
done

args=()
if [ "$select_scope" -eq 1 ]; then
    args+=(--workspace)
fi
args+=("$@")

tmp="$(mktemp "${TMPDIR:-/tmp}/klep-diag.XXXXXX")"
trap 'rm -f "$tmp"' EXIT

cargo "$cargo_cmd" --message-format=json "${args[@]}" >"$tmp"
cargo_rc=$?

python3 - "$tmp" <<'PY'
import json
import sys

path = sys.argv[1]
lines = []
errors = warnings = 0

with open(path, "r", errors="replace") as fh:
    for raw in fh:
        raw = raw.strip()
        if not raw.startswith("{"):
            continue
        try:
            record = json.loads(raw)
        except json.JSONDecodeError:
            continue
        if record.get("reason") != "compiler-message":
            continue

        message = record["message"]
        level = message.get("level")
        if level not in ("error", "warning"):
            continue

        code = message.get("code")
        code = f"[{code['code']}]" if isinstance(code, dict) and code.get("code") else ""

        span = next((s for s in message.get("spans", []) if s.get("is_primary")), None)
        loc = f"{span['file_name']}:{span['line_start']}:{span['column_start']}" if span else "<no span>"
        text = message.get("message", "").splitlines()[0] if message.get("message") else ""

        lines.append(f"{level.upper()}{code} {loc}: {text}")
        if level == "error":
            errors += 1
        else:
            warnings += 1

        for child in message.get("children", []):
            child_text = child.get("message", "").splitlines()
            if child.get("level") in ("error", "warning", "note") and child_text:
                lines.append(f"    ↳ {child_text[0]}")

for line in lines:
    print(line)
sys.stdout.flush()
print(f"{errors} error(s), {warnings} warning(s)", file=sys.stderr)
sys.exit(1 if errors else 0)
PY
diag_rc=$?

if [ "$cargo_rc" -ne 0 ] || [ "$diag_rc" -ne 0 ]; then
    exit 1
fi
exit 0
