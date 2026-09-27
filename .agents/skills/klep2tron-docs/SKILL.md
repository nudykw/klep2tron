---
name: klep2tron-docs
description: Write or update Klep2tron documentation and plans, keeping English/Ukrainian pairs in sync and following the project's product and engineering templates. Use when a code change needs documentation updates, when writing a plan in plans/, or when editing README/docs.
---

# Klep2tron documentation

The project must stay understandable to humans and agents. Documentation is part
of "done", not an afterthought (`docs/AI_WORKFLOW.md` §4).

## Where docs live

| Path | Content |
|---|---|
| `AGENTS.md` | Agent entrypoint (auto-loaded). Keep it short; link, don't duplicate. |
| `README.md` | User-facing overview + install/run for all platforms. EN/UA in one file. |
| `docs/PROJECT_MAP.md` | Technical map of the workspace and key modules. |
| `docs/EDITOR_GUIDE.md` | Map editor controls. |
| `docs/ROOM_RENDERING.md` | Room/tile rendering pipeline. |
| `docs/Actor_Storage_Format.md` | Actor `.k2m` storage format. |
| `docs/design/GDD.md` | Game design. |
| `docs/PRODUCT_PROTOCOL.md` | "What and why" for non-trivial features. |
| `docs/ENGINEERING_PROTOCOL.md` | "How", performance, fast/full path. |
| `docs/SOLID_BEVY.md` | Coding standards (file ≤ 500 lines, plugins, events). |
| `docs/templates/` | `PRODUCT_TEMPLATE.md`, `ENGINEERING_TEMPLATE.md`. |
| `plans/` | Implementation plans, migrations, bug post-mortems. |

## Bilingual rule (mandatory)

Keep the EN/UA pairs synchronised. Editing one requires editing the other:

- `README.md` — English and Ukrainian sections in the same file.
- `docs/AI_WORKFLOW_UA.md` ↔ `docs/AI_WORKFLOW.md`
- `docs/PRODUCT_PROTOCOL_UA.md` ↔ `docs/PRODUCT_PROTOCOL.md`
- `docs/ENGINEERING_PROTOCOL_UA.md` ↔ `docs/ENGINEERING_PROTOCOL.md`

`SOLID_BEVY.md` and the subsystem docs are currently Ukrainian/English mix —
match the language of the file you are editing. Install instructions must cover
**Debian/Ubuntu (`apt`), Arch/Manjaro (`pacman`), Fedora/RHEL (`dnf`), and
macOS (`brew`)**.

## Plans

- Put non-trivial work in `plans/<Descriptive_Name>.md`.
- Start with a status header: `**Статус:** 🔴 ОТКРЫТ / 🟡 В РАБОТЕ / ✅ ВЫПОЛНЕНО`,
  date, branch, component, and links to related docs.
- For features, copy `docs/templates/PRODUCT_TEMPLATE.md` and
  `docs/templates/ENGINEERING_TEMPLATE.md` as required by the protocols.
- Include a Mermaid architecture diagram for system/data-flow changes
  (`ENGINEERING_PROTOCOL.md` §3).
- When a plan finishes, keep it as a post-mortem: what was the root cause, what
  was tried, what actually fixed it, and how it was verified. Good examples:
  `plans/Bevy019_Migration_Plan.md`, `plans/Preview_RTT_Thumbnails_Bug.md`.

## Checklist for a code change

- [ ] If behavior/commands changed, update the doc that describes them
      (`PROJECT_MAP.md`, `EDITOR_GUIDE.md`, `README.md`, subsystem docs).
- [ ] If a public type/format changed, update `Actor_Storage_Format.md` /
      `ROOM_RENDERING.md` as applicable.
- [ ] Keep EN/UA pairs in sync.
- [ ] Add or update a `plans/` document for non-trivial work; record the
      verification method (for UI, the control-API recipes from
      `klep2tron-control`).
- [ ] `AGENTS.md` stays accurate: it should list the crates, commands, and the
      current traps; move growing detail into `docs/` or a skill.
- [ ] Cross-check links after moving files.

## Style

- Prefer short, linkable documents over one giant file.
- Use tables for matrices (crate/target, platform/deps, old/new API).
- Keep code fences runnable and copy-pasteable; use `$PWD`-relative paths.
- Do not duplicate the same matrix in three places — define it once and link.
