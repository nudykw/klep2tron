# 🛠 Convergence Loop: AC-критерии и отчёт о сходимости (по мотивам `github/spec-kit`)

> **Статус:** ✅ ВЫПОЛНЕНО (2026-10-08)
> **Дата:** 2026-10-08
> **Ветка:** `main` (дерево изменено, синк репозитория не выполнен — §2 `docs/AI_WORKFLOW.md`)
> **Компонент:** только документация: `AGENTS.md`, `docs/AI_WORKFLOW{, _UA}.md`,
> `docs/PRODUCT_PROTOCOL{,_UA}.md`, `docs/templates/{PRODUCT,ENGINEERING,BUG}_TEMPLATE.md`
> **Связанные документы:** `docs/AI_WORKFLOW.md` §3.1/§3.2, `docs/PRODUCT_PROTOCOL.md` §3

---

## 1. Цель и проблема

**Проблема.** `verify` проверяет правила и компиляцию, `reviewer` классифицирует нарушения, но ни
один шаг не отвечает на вопрос «всё ли обещанное сделано?». В шаблонах не было секции критериев
приёмки, хотя `docs/PRODUCT_PROTOCOL.md` §3 объявлял их обязательным артефактом; статус
`✅ ВЫПОЛНЕНО` ставился самим исполнителем.

**Цель.** Замкнуть цикл: план → пронумерованные `AC-N` → доказательства → вердикт
`verified | partial | failed` → `Outstanding` для несделанного. Без новых зависимостей и без второго
дерева артефактов.

## 2. Решение

Взято из `github/spec-kit` (вариант A — идеи, а не установка тулкита):

* оракул завершения (`AC-N`) + отчёт сходимости вместо второго мнения LLM;
* вердикты баг-процесса `verified | partial | failed` и запрет закрывать баг без повторного прогона
  симптома;
* явный раздел `Outstanding` вместо молчаливого выпадения требования.

Отвергнуто: `.specify/` как второе дерево артефактов (дубль `plans/`); `constitution.md` (дубль
`AGENTS.md` + `docs/SOLID_BEVY.md`); `taskstoissues` (в Pi нет MCP).

Сильная сторона по сравнению с оригиналом: критерии формулируются как **наблюдения, проверяемые
control-API и `preview_check`** (`docs/AI_WORKFLOW.md` §3.1), то есть сходимость подтверждается
артефактами, а не перечитыванием спеки той же моделью.

## 3. Критерии приёмки (AC)

*   **AC-1:** Протоколы требуют нумерованные `AC-N` / `T-N` — **проверка:** `read docs/PRODUCT_PROTOCOL.md` §3 и `_UA` §3.
*   **AC-2:** Шаблоны содержат AC / Task Breakdown / Converge-отчёт / Outstanding — **проверка:** `read docs/templates/{PRODUCT,ENGINEERING,BUG}_TEMPLATE.md`.
*   **AC-3:** Правила сходимости и баг-формата зафиксированы и не противоречат `verify`-workflow — **проверка:** `docs/AI_WORKFLOW.md` §3.1–3.2, `AGENTS.md`.
*   **AC-4:** UA↔EN пары синхронны (§4 `docs/AI_WORKFLOW.md`) — **проверка:** сверка секций `AI_WORKFLOW{, _UA}.md` и `PRODUCT_PROTOCOL{,_UA}.md`.
*   **AC-5:** Код не изменён и компилируется — **проверка:** `verify(mode="rust")`.
*   **AC-6:** Правила репозитория без новых нарушений — **проверка:** `verify(mode="rules")`.

## 4. Разбиение задач (Task Breakdown)

*   [x] **T-1** [`AC-1`, `AC-2`] — шаблоны PRODUCT / ENGINEERING (§7–§9, §7)
*   [x] **T-2** [`AC-3`] — `AI_WORKFLOW` §3.1/§3.2 (+ UA)
*   [x] **T-3** [`AC-1`] — `PRODUCT_PROTOCOL` §3 (+ UA)
*   [x] **T-4** [`AC-3`] — `AGENTS.md`, блок Verification workflow
*   [x] **T-5** [`AC-2`] — `BUG_TEMPLATE.md` + ретро-эталон `plans/Preview_RTT_Thumbnails_Bug.md`

## 5. Отчёт о сходимости (Converge)

| AC | Статус | Доказательство (что реально прогнали) | Вердикт |
|----|--------|--------------------------------------|---------|
| AC-1 | ✅ | `docs/PRODUCT_PROTOCOL.md` §3: `AC-1, AC-2, …` и `T-1, T-2, …`; то же в `_UA.md` §3 | verified |
| AC-2 | ✅ | `PRODUCT_TEMPLATE.md` §7–§9 (AC, Task Breakdown, таблица Converge), `ENGINEERING_TEMPLATE.md` §7 (Outstanding), `BUG_TEMPLATE.md` §0/§3 | verified |
| AC-3 | ✅ | `AI_WORKFLOW.md` §3.1 (5 шагов) и §3.2 (3 шага); `AGENTS.md` → «Then run the Convergence Check…» | verified |
| AC-4 | ✅ | `git diff --stat`: `AI_WORKFLOW.md` +20 / `AI_WORKFLOW_UA.md` +20; `PRODUCT_PROTOCOL.md` 4 / `_UA.md` 4 — правки парные | verified |
| AC-5 | ✅ | `verify(mode="rust")` → **PASS**, `cargo check --workspace` exit=0 (`~/.cache/pi-verify/verify-20261008-205404-rust.log`) | verified |
| AC-6 | ✅ | `verify(mode="rules")` → REVIEW: 13× `GlobalZIndex` + 2× формула тайла; `git diff --name-only` = только `*.md`, т.е. кандидаты предсуществующие. Классификация вручную: 10 легитимных оверлеев/тултипов + 3 строки комментария, формула — внутри `rendering/tile_geometry.rs`. Нарушений 0 | verified (классификация ручная) |

**Вердикт по задаче:** `verified`.

## 6. Outstanding (сознательно НЕ сделано)

*   **Автоматического enforcement нет** — отчёт заполняет исполнитель, проверяет человек. Follow-up: правило в `.pi/verify-rules.txt` или шаг в `verify`, требующий наличия таблицы Converge в плане.
*   **Прогон через `reviewer` не выполнен** — инструмент `subagent` в этой сессии не подключён; классификация кандидатов `verify(rules)` сделана вручную (штатный fallback из skill `verify-change`), поэтому вердикт по AC-6 помечен как ручной.
*   **Ретрофит старых планов** — сделан только для `plans/Preview_RTT_Thumbnails_Bug.md` (как эталон баг-формата). Остальные планы закрыты и не переразмечались, чтобы не переписывать историю.
