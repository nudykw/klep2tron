# Dev tooling — LSP config + structured diagnostics

Status: implemented (2026-09-26)

## Context

The repo had **no** editor/LSP configuration. rust-analyzer is used via the VS
Code extension (process-level), with defaults. The agent (pi) has no LSP tool;
it works from `read`/`rg`/`cargo`. Two cheap improvements were agreed:

1. a root `rust-analyzer.toml` tuning RA for this Bevy workspace;
2. a reproducible way to read `cargo check --message-format=json` output as
   compact, structured diagnostics.

## Facts established from the rust-analyzer sources (v0.3.3057)

- RA reads `rust-analyzer.toml` from the project root **and** per-crate
  (`**/rust-analyzer.toml`); the root file is parsed as
  `WorkspaceLocalConfigInput`.
- Keys are nested TOML tables derived from the config key
  (`field.replace('_', '/')`, e.g. `check_command` -> `[check] command`).
- **Only `workspace`/`local` scoped keys are valid at the root.** `global`
  options (`files.exclude`, `procMacro.*`, `cachePriming.*`) are *not* readable
  from the project-root file — those need the user config
  (`~/.config/rust-analyzer/rust-analyzer.toml`). Do not put them here; they
  would be silently ignored / reported as unexpected fields.
- `target/` is ~93 GB (Bevy debug). A separate `cargo.targetDir` for RA would
  roughly duplicate that, so it is deliberately **not** set.

## 1. `rust-analyzer.toml` (project root, workspace scope)

```toml
[check]
command = "clippy"     # project already lints with clippy
allTargets = true      # also cover tests/benches (mirrors `cargo test --no-run`)
```

Leave `check.workspace` at its default (whole workspace) for cross-crate
errors; set it to `false` only if per-save latency becomes a problem.

## 2. `scripts/diagnostics.sh`

Wrapper around `cargo {check|clippy} --workspace --message-format=json` that
prints `LEVEL[code] file:line:col: message` (plus nested notes) and a summary,
and exits non-zero when there are errors. `CARGO_CMD=clippy` switches linter;
extra args are forwarded (e.g. `-p editor_client`).

This is the agent-facing, credit-cheap alternative to full `cargo check` text
output and replaces guessing from truncated logs.

## Verification

- [ ] `python3 -c "import tomllib; tomllib.load(open('rust-analyzer.toml','rb'))"`
- [ ] keys exist among RA `WorkspaceConfigInput`/`LocalConfigInput` fields.
- [ ] `scripts/diagnostics.sh -p shared` runs and exits 0 on a clean crate.
- [ ] Full `cargo check --workspace` still green.

## Non-goals

- No global user config changes (`~/.config/rust-analyzer/`).
- No `xtask` crate, no CI changes.

## Follow-up — workspace is clippy-clean (2026-09-26)

Enabling clippy exposed 93 warnings (invisible before, since the project never
ran clippy). Resolution:

- `clippy.toml` raises `too-many-arguments-threshold = 16` and
  `type-complexity-threshold = 1000` for the Bevy-idiomatic lints instead of
  littering systems with `#[allow]` (~54 warnings gone).
- The remaining ~39 were fixed (`cargo clippy --fix` for the machine-applicable
  ones + manual edits): `is_none_or`/`is_some_and`, `Range::contains`,
  `matches!`, `clamp`, iterator loops, collapsed `if`s, merged identical
  `if_same_then_else` branches, etc. Behaviour preserved.
- Result: `cargo clippy --workspace --all-targets`, `cargo check --workspace`
  and `cargo test --workspace --no-run` all clean.

Note: the repo is **not** `rustfmt`-clean (hundreds of pre-existing diffs), so
`cargo fmt` was deliberately **not** run.
