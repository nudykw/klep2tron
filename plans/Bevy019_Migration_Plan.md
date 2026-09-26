# 🛠 Миграция Klep2tron на Bevy 0.19

> **Статус:** ✅ Выполнено (2026-09-26) — весь workspace компилируется под Bevy 0.19
> (native + wasm), `client` и `editor_client` запускаются. Остался один визуальный
> баг с RTT-превью: [Preview_RTT_Thumbnails_Bug.md](Preview_RTT_Thumbnails_Bug.md).
> Дата: 2026-09-26
> **Тип задачи:** Инфраструктурная миграция (не фича)
> **Основание:** результаты spike `spike/bevy-0.19` (2026-09-26)
> **Связанные документы:** [PROJECT_MAP.md](../docs/PROJECT_MAP.md), [GDD.md](../docs/design/GDD.md), [ENGINEERING_PROTOCOL.md](../docs/ENGINEERING_PROTOCOL.md), [Extraction_Plan.md](./ActorEditor/Extraction_Plan.md)

---

## 1. Зачем это нужно (и почему именно сейчас)

Проект на **Bevy 0.14.2**. Библиотеки, прописанные в GDD для игровой части, существуют **только под Bevy 0.19**:

| Библиотека (GDD) | Актуальная | Требует Bevy |
|---|---|---|
| Avian3D (физика) | 0.7.0 | `^0.19.0` |
| bevy_renet (сеть) | 5.0.0 | `^0.19` |
| leafwing-input-manager (управление) | 0.21.0 | `^0.19` |

Редакторные плагины (`bevy_obj`, `bevy_panorbit_camera`, `bevy_hanabi`, `bevy_framepace`) тоже ушли на 0.19. `bevy_mod_picking` — **мёртвый крейт** (последняя версия 0.20.1 под Bevy 0.14); в Bevy 0.15+ picking встроен в движок.

**Вывод:** оставаясь на 0.14, невозможно реализовать игровую часть по GDD. При этом игрового кода ещё нет — **сейчас миграция дешевле, чем после написания физики/сети поверх 0.14.**

---

## 2. Целевая версия и матрица зависимостей

**Цель: Bevy 0.19.1** (единственная версия, где есть все нужные крейты).

| Пакет | Сейчас | Цель | Примечание |
|---|---|---|---|
| bevy | 0.14.2 | 0.19.1 | |
| bevy_obj | 0.14.0 | 0.19.0 | |
| bevy_panorbit_camera | 0.20.1 | 0.35.1 | |
| bevy_hanabi | 0.13.1 | 0.19.0 | |
| bevy_framepace | 0.17.2 | 0.22.0 | |
| **bevy_mod_picking** | 0.20.1 | **удалить** | замена на встроенный `bevy_picking` |
| wgpu | 0.20.1 | 29.x | задаётся Bevy (`bevy_render` требует `^29.0.3`) |
| winit | 0.30.13 | 0.30.x | без изменений |

---

## 3. Измеренная поверхность миграции (данные spike)

Spike: `bevy 0.19.1` + плагины 0.19 + `wgpu 29`, `cargo check --workspace`.
До остальных крейтов сборка не дошла — **`client_core` дал 207 ошибок**, вызванных **~23 уникальными символами** (высокий fan-out, низкое разнообразие → правится механически).

### Карта замен

| Было | Стало | Вхождений (workspace) |
|---|---|---|
| `Style` | `Node` | 202 |
| `TextStyle` | `TextFont` + `TextColor` | 123 |
| `NodeBundle` | `Node` (required components) | 117 |
| `TextBundle` | `Text` | 117 |
| `Query::get_single` | `Query::single` | 93 |
| `Query::get_single_mut` | `Query::single_mut` | 35 |
| `despawn_recursive()` | `despawn()` (каскад по умолчанию) | 23 |
| `PbrBundle` | компоненты напрямую | 19 |
| `Parent` | `ChildOf` | 15 |
| `Camera3dBundle` / `Camera2dBundle` | компоненты напрямую | 9 |
| `DirectionalLightBundle` | компоненты напрямую | 6 |
| `Time::delta_seconds` / `elapsed_seconds` | `delta_secs` / `elapsed_secs` | 9 |
| `GamepadButtonType` | переименован | 8 |
| `JustifyText`, `FogSettings`, `ChildBuilder`, `apply_deferred`, `NotShadowCaster/Receiver`, `MaterialMeshBundle` | переименованы/удалены | по 1–5 |
| `bevy_mod_picking::*` | `bevy_picking` (встроенный) | 14 |
| Render: `ScreenSpaceAmbientOcclusion*`, `bevy::core_pipeline::bloom`, `ShaderRef` | переехали/переименованы | ~10 |

**Итого ~500+ прямых обращений**, из них ~90% — механические замены.

---

## 4. Пакеты работ (Work Packages)

### WP1 — Инфраструктура и манифесты
- Обновить версии во всех `Cargo.toml` (см. матрицу §2).
- `wgpu` → 29 (ведомый за Bevy, иначе две несовместимые копии).
- Убрать `bevy_mod_picking` из `actor_editor`.
- Зафиксировать целевой `Cargo.lock` (он теперь под git — хорошо).
- **Критерий:** `cargo metadata` резолвится, крейты без Bevy-кода (`shared`, `server`) собираются.

### WP2 — ECS и иерархия
- `Query::get_single/get_single_mut` → `single/single_mut` (учть: возвращают `Result`, поправить обработку).
- `Parent` → `ChildOf`; пересмотреть `Children`-итерации.
- `despawn_recursive()` → `despawn()`.
- `apply_deferred` → встроено/`ApplyDeferred`.
- **Критерий:** `client_core` компилируется без ошибок этой группы.

### WP3 — UI (самый большой объём: ~450 вхождений)
- `Style` → `Node`.
- `NodeBundle` → `Node` (+ связанные компоненты: `BackgroundColor`, `BorderColor`, `ZIndex` и т.д.).
- `TextBundle` → `Text`; `TextStyle` → `TextFont`/`TextColor`; `JustifyText` → актуальное.
- `ChildBuilder` → `ChildSpawnerCommands`.
- **Критерий:** меню, HUD, инспектор и панели редактора рисуются корректно (smoke).

### WP4 — Render
- SSAO (`ScreenSpaceAmbientOcclusionBundle/Settings`) — новый путь.
- `bevy::core_pipeline::bloom` — новый путь.
- `ShaderRef` — новый путь; проверить `starry_sky.wgsl` (кастомный материал).
- Свет/тени: `DirectionalLightBundle`, `DirectionalLightShadowMap`, `NotShadowCaster/Receiver`, `CascadeShadowConfigBuilder`.
- `FogSettings` → актуальный туман.
- **Критерий:** сцена рендерится, туман/свет/скайбокс на месте.

### WP5 — Picking (единственная семантическая замена)
- `bevy_mod_picking` → встроенный `bevy_picking`.
- Мигрировать: `PickableBundle`/`Pickable`, `Pointer<Click>` и прочие события, `DefaultPickingPlugins`.
- Затронуто: выделение сокетов, гизмо, ласо, `slicing`, `picking`, `gizmos/*`.
- **Критерий:** выделение треугольников, перетаскивание сокетов, ласо-выделение работают; инспекция реагирует на клики.

### WP6 — Прочее
- Ввод: `GamepadButtonType`, `Gamepads`.
- Время: `delta_secs`, `elapsed_secs`, `Timer::is_finished`.
- Материалы/меши: `MaterialMeshBundle`, кастомные `Material` трейты (проверить сигнатуры).

### WP7 — Верификация и документация
- Smoke-цикл редактора (см. §6).
- Проверка wasm (`client_web`, `editor_client_web`).
- Обновить `PROJECT_MAP.md`, `README.md` (версия движка), при необходимости — протоколы.

---

## 5. Стратегия версий: поэтапно или одним проходом

| Подход | Плюсы | Минусы |
|---|---|---|
| **A. Поэтапно** 0.14→0.15→…→0.19 | у каждого шага есть официальный migration guide; ловишь регрессии на границе версий | 5 раундов правок одних и тех же файлов; дольше |
| **B. Одним проходом** 0.14→0.19 | один проход по коду, минимум повторной работы | нет единого migration guide; сложнее локализовать причину регрессии |

**Рекомендация: гибрид.** Сделать **0.14→0.15 отдельным этапом** (самый крупный: required components + переписанный UI/текст + встроенный picking — это ~80% объёма), а затем **0.15→0.19 одним этапом**, где правки в основном точечные. Так получаем один «тяжёлый» референсный переход и один «добивочный», без пяти раундов.

**Порядок крейтов:** `shared` → `server` → `client_core` → `actor_editor` → `client`/`editor_client` → wasm-обёртки. Крупный `actor_editor` проверяется отдельно.

---

## 6. Критерии готовности (smoke)

После каждого этапа — обязательно:

1. `cargo check --workspace` — чисто.
2. `cargo check -p client_web --target wasm32-unknown-unknown` и `editor_client_web` — чисто.
3. `cargo run -p editor_client` — полный цикл NPC-редактора:
   **импорт модели → оптимизация → разрезание → сокеты → сохранение `.k2m` → загрузка**.
4. `cargo run -p client` — меню и переход в игру (START GAME) без паник.
5. Ручная проверка render: туман, тени, скайбокс, тени SSAO/bloom.
6. Правка/создание карты в `editor_client` (RTT-превью тайлов, орбитальная камера).

**Откат:** миграция идёт на отдельной ветке (`feat/bevy-0.19`) от `feat/actor-editor`; `feat/actor-editor` не трогается до финала.

---

## 7. Риски и дегенеративные случаи

| Риск | Вероятность | Митигация |
|---|---|---|
| `bevy_mod_picking` → `bevy_picking`: смена модели событий ломает выделение/гизмо | **Высокая** | отдельный WP5; smoke каждого сценария выделения; при необходимости — временно вернуть свою систему рейкаста |
| Render-API (SSAO/bloom/ShaderRef/fog) — не только имена, но и поведение | Средняя | отдельный WP4; визуальная проверка на скриншотах до/после |
| Тихое изменение поведения UI (required components) — вёрстка «плывёт» | Средняя | визуальный smoke меню/инспектора; сравнение скриншотов |
| Огромный diff → конфликты при параллельной работе | Средняя | заморозка фич редактора на время миграции; одна ветка, один автор |
| `meshopt`/`rfd`/`ron` несовместимы с новым Bevy | Низкая | уже обновлены; проверить на этапе WP1 |
| Кастомный `.k2m`/bake-пайплайн зависит от старого API мешей | Средняя | тест сохранения/загрузки актора в smoke |

---

## 8. Оценка

| | |
|---|---|
| Объём | ~500+ прямых правок + render/picking-резидуум |
| Трудоёмкость | **1.5–3 дня** сфокусированной работы |
| Сложность | низкая/средняя (в основном механические замены), кроме WP4/WP5 |
| Зависимости | заморозка фич редактора (задачи 11 и 14.5) на время миграции |

### Последовательность (предлагаемая)

1. Утвердить план, создать ветку `feat/bevy-0.19`, заморозить фичи.
2. WP1 (инфраструктура) → WP2 → WP3 → WP4 → WP5 → WP6.
3. Этап 0.14→0.15, smoke.
4. Этап 0.15→0.19, smoke.
5. WP7 (документация), финальный smoke, merge в `feat/actor-editor`.

---

> **Примечание для исполнителя (ИИ):** конкретные имена API для 0.17–0.19 берутся из официальных Bevy migration guides соответствующей версии в момент выполнения — этот документ фиксирует **объём и стратегию**, а не является заменой migration guide. Символьная карта в §3 подтверждена компилятором на целевом 0.19.
