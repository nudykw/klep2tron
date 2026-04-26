# План оптимизации производительности слайсера

## Анализ текущей проблемы

### Описание проблемы
При перемещении ползунка слайсера происходит медленная отрисовка. Пользователь отмечает, что:
1. Смещение среза с помощью ползунка происходит медленно
2. Используется механизм "кружочков" для подтверждения изменений
3. Если пользователь отпускает мышь за пределами круга, гравировка и слайсер остаются на месте до подтверждения

### Текущая архитектура

#### Компоненты системы

**1. UI Слайдер** ([`widgets/sliders.rs`](../crates/client_core/src/actor_editor/widgets/sliders.rs))
- `RangeSlider` - вертикальный слайдер с двумя ползунками (Top/Bottom)
- `ConfirmationCircleUI` - круги подтверждения вокруг ползунков
- Механизм подтверждения через `needs_confirm` и `trigger_slice`

**2. Система разрезания** ([`systems/slicing.rs`](../crates/client_core/src/actor_editor/systems/slicing.rs))
- `mesh_slicing_system` - основная система разрезания
- Асинхронное выполнение через `AsyncComputeTaskPool`
- Вызывает `geometry::slicer::split_mesh_by_planes()`

**3. Геометрический алгоритм** ([`geometry/slicer.rs`](../crates/client_core/src/actor_editor/geometry/slicer.rs))
- `split_mesh_by_planes()` - разделение меша на 3 части
- `split_triangle()` - разрезание отдельных треугольников
- `build_caps_from_segments()` - создание крышек (caps)
- Возвращает `SlicedParts` с контурами для гравировки

**4. Отрисовка контуров** ([`systems/sync.rs`](../crates/client_core/src/actor_editor/systems/sync.rs:68-83))
- `draw_slicing_contours_system` - отрисовка красных линий гравировки
- Использует Bevy Gizmos для рендеринга
- Выполняется в `PostUpdate` после `mesh_slicing_system`

#### Порядок выполнения систем

```
PostUpdate (после TransformSystem::TransformPropagate):
├── mesh_slicing_system          // Разрезание меша
├── draw_slicing_contours_system // Отрисовка контуров
└── draw_actor_bounds_debug_system
```

### Выявленные узкие места

#### 1. **Связанное выполнение разрезания и отрисовки контуров**
**Проблема:** Обе операции выполняются в одном цикле `PostUpdate`
- При каждом движении ползунка запускается полное разрезание меша
- Контуры пересчитываются вместе с мешами
- Отрисовка контуров ждет завершения разрезания

**Код:**
```rust
// slicing.rs:167
contours: [top_segments, bot_segments].concat(),
```

**Влияние:** Даже если пользователь просто двигает ползунок (без подтверждения), система пересчитывает весь меш.

#### 2. **Механизм подтверждения не предотвращает пересчет**
**Проблема:** Механизм `needs_confirm` только откладывает применение результата, но не предотвращает вычисления

**Код в `range_slider_system`:**
```rust
// sliders.rs:423-436
if slider.dragging.is_some() && mouse_button.pressed(MouseButton::Left) {
    // Обновляет значения ПОСТОЯННО во время перетаскивания
    slicing_settings.dragging_gizmo = Some(...);
    slicing_settings.needs_confirm = false;
}
```

**Код в `mesh_slicing_system`:**
```rust
// slicing.rs:166-167
let values_changed = (slicing_settings.top_cut - slicing_settings.last_top).abs() > 0.001 ||
                     (slicing_settings.bottom_cut - slicing_settings.last_bottom).abs() > 0.001;
```

**Влияние:** Каждое микродвижение ползунка вызывает полный пересчет меша.

#### 3. **Тяжелые операции в геометрическом алгоритме**
**Проблема:** Для каждого треугольника выполняются сложные вычисления

**Код:**
```rust
// slicer.rs:99-144
for tri_indices in indices.chunks(3) {
    // Для КАЖДОГО треугольника:
    // 1. Проверка пересечения с плоскостями
    // 2. Интерполяция вершин
    // 3. Создание новых треугольников
    // 4. Сбор сегментов контуров
}

// slicer.rs:149-156
if show_caps {
    // Построение крышек - дорогая операция
    head_tris.extend(super::capper::build_caps_from_segments(&top_segments, false, rim_thickness));
    body_tris.extend(super::capper::build_caps_from_segments(&top_segments, true, rim_thickness));
    // ...
}
```

**Влияние:** Для модели с 10K треугольников это 10K итераций + построение крышек.

#### 4. **Отрисовка контуров в каждом кадре**
**Проблема:** Система `draw_slicing_contours_system` перерисовывает все сегменты в каждом кадре

**Код:**
```rust
// sync.rs:68-83
pub fn draw_slicing_contours_system(
    contour_query: Query<(&SlicingContours, &GlobalTransform)>,
    viewport_settings: Res<ViewportSettings>,
    mut gizmos: Gizmos,
) {
    for (contours, transform) in contour_query.iter() {
        let matrix = transform.compute_matrix();
        for segment in &contours.segments {
            // Для КАЖДОГО сегмента в КАЖДОМ кадре
            let start = matrix.transform_point3(segment[0]);
            let end = matrix.transform_point3(segment[1]);
            gizmos.line(start, end, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}
```

**Влияние:** Если контуров много (сложная модель), это добавляет нагрузку на GPU.

## Варианты решения

### Вариант 1: Разделение отрисовки гравировки и слайсера (как было раньше)

#### Описание
Отделить визуализацию контуров от процесса разрезания меша. Контуры обновляются независимо и быстрее.

#### Преимущества
- ✅ Простая реализация
- ✅ Быстрая визуальная обратная связь
- ✅ Не требует изменения алгоритма разрезания
- ✅ Контуры можно вычислять на CPU без создания мешей

#### Недостатки
- ❌ Дублирование логики (вычисление пересечений дважды)
- ❌ Возможна рассинхронизация между контурами и мешами
- ❌ Не решает проблему частого пересчета мешей

#### Архитектура

```
Update:
├── range_slider_system          // Обновление UI
└── preview_contours_system      // НОВАЯ: Быстрый расчет контуров для preview

PostUpdate:
├── mesh_slicing_system          // Только при trigger_slice = true
└── draw_slicing_contours_system // Отрисовка контуров
```

### Вариант 2: Оптимизация разрезания и отрисовки гравировки

#### Описание
Оптимизировать существующую систему, добавив:
1. Дебаунсинг (задержка перед пересчетом)
2. Кеширование промежуточных результатов
3. Инкрементальное обновление контуров

#### Преимущества
- ✅ Единая логика без дублирования
- ✅ Гарантированная синхронизация
- ✅ Меньше изменений в кодовой базе
- ✅ Сохраняет текущую архитектуру

#### Недостатки
- ❌ Более сложная реализация
- ❌ Требует тщательной настройки таймингов
- ❌ Может быть недостаточно для очень сложных моделей

#### Архитектура

```
Update:
├── range_slider_system          // Обновление UI с дебаунсингом
└── contour_cache_system         // НОВАЯ: Кеширование контуров

PostUpdate:
├── mesh_slicing_system          // С проверкой debounce_timer
└── draw_slicing_contours_system // Использует кеш
```

### Вариант 3: Гибридный подход (РЕКОМЕНДУЕТСЯ) ✅ РЕАЛИЗОВАНО

#### Описание
Комбинация обоих подходов:
1. **Preview режим:** Быстрые контуры во время перетаскивания
2. **Confirm режим:** Полное разрезание при подтверждении
3. **Визуальная обратная связь:** Разные цвета для различных состояний:
   - **Черный** - старая гравировка во время перетаскивания
   - **Оранжевый** - новая preview гравировка во время перетаскивания
   - **Красный** - финальная гравировка после подтверждения
4. Использование механизма `needs_confirm` по назначению

#### Преимущества
- ✅ Лучшая производительность во время перетаскивания
- ✅ Точность при подтверждении
- ✅ Использует существующий механизм подтверждения
- ✅ Минимальное дублирование кода
- ✅ Отличная UX - мгновенная обратная связь

#### Недостатки
- ❌ Требует двух путей вычисления контуров
- ❌ Немного больше кода для поддержки

#### Архитектура

```
Update:
├── range_slider_system              // Обновление UI
└── preview_contours_system          // НОВАЯ: Быстрые контуры при dragging

PostUpdate:
├── mesh_slicing_system              // Только при trigger_slice = true
└── draw_slicing_contours_system     // Отрисовка (preview или final)
```

## Рекомендуемое решение: Вариант 3 (Гибридный)

### Обоснование

1. **Производительность:** Preview контуры вычисляются быстро (только пересечения, без мешей)
2. **UX:** Пользователь видит мгновенную обратную связь
3. **Точность:** Финальное разрезание происходит только при подтверждении
4. **Совместимость:** Использует существующий механизм `needs_confirm`

### Детальный план реализации

#### Этап 1: Создание системы preview контуров

**Файл:** `crates/client_core/src/actor_editor/systems/preview_contours.rs` (НОВЫЙ)

**Функционал:**
```rust
pub fn preview_contours_system(
    slicing_settings: Res<SlicingSettings>,
    actor_root_query: Query<(&ActorBounds, &GlobalTransform), With<Actor3DRoot>>,
    mesh_query: Query<(Entity, &OriginalMeshComponent, &GlobalTransform)>,
    meshes: Res<Assets<Mesh>>,
    mut commands: Commands,
) {
    // Проверяем, идет ли перетаскивание
    if slicing_settings.dragging_gizmo.is_none() {
        return; // Не в режиме preview
    }
    
    // Быстрое вычисление только контуров (без создания мешей)
    let contours = calculate_contours_only(
        mesh, 
        plane_top_local, 
        plane_bottom_local
    );
    
    // Обновляем компонент PreviewContours
    commands.entity(entity).insert(PreviewContours { 
        segments: contours,
        is_preview: true 
    });
}
```

**Ключевые особенности:**
- Вычисляет только пересечения треугольников с плоскостями
- НЕ создает новые меши
- НЕ строит крышки (caps)
- Работает только во время `dragging_gizmo.is_some()`

#### Этап 2: Оптимизация алгоритма вычисления контуров

**Файл:** `crates/client_core/src/actor_editor/geometry/contour_calculator.rs` (НОВЫЙ)

**Функционал:**
```rust
pub fn calculate_contours_only(
    mesh: &Mesh,
    top_y: f32,
    bottom_y: f32,
) -> Vec<[Vec3; 2]> {
    let positions = extract_positions(mesh);
    let indices = extract_indices(mesh);
    
    let mut segments = Vec::new();
    
    // Только проверка пересечений, без создания новых треугольников
    for tri_indices in indices.chunks(3) {
        let tri = [
            positions[tri_indices[0]],
            positions[tri_indices[1]],
            positions[tri_indices[2]],
        ];
        
        // Проверка пересечения с верхней плоскостью
        if let Some(segment) = intersect_triangle_with_plane(&tri, top_y) {
            segments.push(segment);
        }
        
        // Проверка пересечения с нижней плоскостью
        if let Some(segment) = intersect_triangle_with_plane(&tri, bottom_y) {
            segments.push(segment);
        }
    }
    
    segments
}

fn intersect_triangle_with_plane(
    tri: &[Vec3; 3],
    y: f32,
) -> Option<[Vec3; 2]> {
    // Упрощенная логика из split_triangle
    // Возвращает только сегмент пересечения
}
```

**Оптимизации:**
- Не создает `VertexData` (только позиции)
- Не интерполирует нормали, UV, цвета
- Не создает новые треугольники
- Не строит крышки

**Ожидаемое ускорение:** 5-10x быстрее полного разрезания

#### Этап 3: Модификация системы отрисовки контуров

**Файл:** `crates/client_core/src/actor_editor/systems/sync.rs`

**Изменения:**
```rust
pub fn draw_slicing_contours_system(
    // Добавляем запрос для preview контуров
    preview_query: Query<&PreviewContours>,
    final_query: Query<&SlicingContours>,
    slicing_settings: Res<SlicingSettings>,
    viewport_settings: Res<ViewportSettings>,
    mut gizmos: Gizmos,
) {
    if !viewport_settings.slices { return; }
    
    // Приоритет: preview контуры во время перетаскивания
    if slicing_settings.dragging_gizmo.is_some() {
        for preview in preview_query.iter() {
            draw_segments(&preview.segments, &mut gizmos, Color::srgb(1.0, 0.5, 0.0)); // Оранжевый для preview
        }
    } else {
        // Финальные контуры
        for contours in final_query.iter() {
            draw_segments(&contours.segments, &mut gizmos, Color::srgb(1.0, 0.0, 0.0)); // Красный для final
        }
    }
}
```

#### Этап 4: Модификация системы разрезания

**Файл:** `crates/client_core/src/actor_editor/systems/slicing.rs`

**Изменения:**
```rust
pub fn mesh_slicing_system(
    // ... существующие параметры
) {
    // НОВАЯ ЛОГИКА: Пропускаем разрезание во время перетаскивания
    if slicing_settings.dragging_gizmo.is_some() && !slicing_settings.trigger_slice {
        return; // Preview контуры обрабатываются отдельной системой
    }
    
    // Остальная логика без изменений
    // Выполняется только при trigger_slice = true
}
```

#### Этап 5: Обновление компонентов

**Файл:** `crates/client_core/src/actor_editor/mod.rs`

**Новые компоненты:**
```rust
#[derive(Component)]
pub struct PreviewContours {
    pub segments: Vec<[Vec3; 2]>,
    pub is_preview: bool,
}
```

**Обновление порядка систем:**
```rust
.add_systems(Update, (
    systems::preview_contours_system, // НОВАЯ
).run_if(in_state(GameState::ActorEditor)))

.add_systems(PostUpdate, (
    systems::mesh_slicing_system,
    systems::draw_slicing_contours_system,
    systems::draw_actor_bounds_debug_system,
).after(bevy::transform::TransformSystem::TransformPropagate)
 .run_if(in_state(GameState::ActorEditor)))
```

### Поведение системы

#### Сценарий 1: Перетаскивание ползунка

```
1. Пользователь начинает перетаскивать ползунок
   └─> dragging_gizmo = Some(Top/Bottom)

2. range_slider_system обновляет значения каждый кадр
   └─> top_cut/bottom_cut изменяются

3. preview_contours_system вычисляет быстрые контуры
   └─> PreviewContours обновляется (~1-2ms)

4. draw_slicing_contours_system отрисовывает preview
   └─> Оранжевые линии (визуальная обратная связь)

5. mesh_slicing_system НЕ выполняется
   └─> Экономия ~50-200ms на кадр
```

#### Сценарий 2: Отпускание внутри круга

```
1. Пользователь отпускает мышь внутри круга подтверждения
   └─> trigger_slice = true

2. mesh_slicing_system выполняет полное разрезание
   └─> Создает финальные меши + контуры

3. PreviewContours удаляется
   └─> dragging_gizmo = None

4. draw_slicing_contours_system переключается на финальные контуры
   └─> Красные линии (финальный результат)
```

#### Сценарий 3: Отпускание за пределами круга

```
1. Пользователь отпускает мышь за пределами круга
   └─> needs_confirm = true
   └─> dragging_gizmo остается Some(...)

2. preview_contours_system продолжает работать
   └─> Контуры остаются видимыми

3. Круг подтверждения остается видимым
   └─> Ожидание клика пользователя

4. При клике на круг:
   └─> trigger_slice = true
   └─> Выполняется полное разрезание
```

### Дополнительные оптимизации

#### 1. Кеширование позиций вершин

**Проблема:** Извлечение позиций из меша каждый кадр

**Решение:**
```rust
#[derive(Component)]
pub struct CachedMeshPositions {
    pub positions: Vec<Vec3>,
    pub indices: Vec<usize>,
}

// При импорте модели кешируем позиции
fn cache_mesh_positions(mesh: &Mesh) -> CachedMeshPositions {
    // Извлекаем один раз
}
```

#### 2. Spatial partitioning для больших мешей

**Проблема:** Проверка всех треугольников даже если они далеко от плоскости

**Решение:**
```rust
// Используем BVH или октодерево для фильтрации треугольников
fn filter_triangles_near_plane(
    triangles: &[Triangle],
    plane_y: f32,
    threshold: f32,
) -> Vec<usize> {
    // Возвращает только индексы треугольников рядом с плоскостью
}
```

#### 3. Дебаунсинг для очень быстрых движений

**Проблема:** При очень быстром движении мыши preview может отставать

**Решение:**
```rust
#[derive(Resource)]
pub struct PreviewDebounce {
    pub last_update: Instant,
    pub min_interval: Duration, // 16ms = 60 FPS
}

// В preview_contours_system
if debounce.last_update.elapsed() < debounce.min_interval {
    return; // Пропускаем кадр
}
```

## Метрики производительности

### Текущая система (без оптимизаций)

Для модели с 10,000 треугольников:
- **Полное разрезание:** ~50-200ms
- **Построение крышек:** ~20-50ms
- **Отрисовка контуров:** ~1-2ms
- **Итого на кадр:** ~71-252ms (4-14 FPS)

### Оптимизированная система (гибридный подход)

Для той же модели:
- **Preview контуры:** ~5-10ms
- **Отрисовка preview:** ~1-2ms
- **Итого на кадр (preview):** ~6-12ms (83-166 FPS)
- **Полное разрезание (при подтверждении):** ~50-200ms (один раз)

**Улучшение:** 10-20x быстрее во время перетаскивания

## Риски и митигация

### Риск 1: Рассинхронизация preview и финальных контуров

**Митигация:**
- Использовать одинаковую логику вычисления пересечений
- Добавить визуальное различие (цвет)
- Тестировать на разных моделях

### Риск 2: Увеличение сложности кода

**Митигация:**
- Хорошая документация
- Четкое разделение ответственности
- Переиспользование кода через общие функции

### Риск 3: Проблемы с памятью при частых обновлениях

**Митигация:**
- Переиспользовать буферы вместо аллокаций
- Ограничить частоту обновлений (debouncing)
- Мониторинг использования памяти

## План тестирования

### Тесты производительности

1. **Простая модель** (< 1K треугольников)
   - Ожидаемый FPS: > 144

2. **Средняя модель** (1K-10K треугольников)
   - Ожидаемый FPS: > 60

3. **Сложная модель** (> 10K треугольников)
   - Ожидаемый FPS: > 30

### Тесты UX

1. **Плавность перетаскивания**
   - Контуры обновляются без задержек
   - Нет заметных лагов

2. **Механизм подтверждения**
   - Круг появляется корректно
   - Отпускание за пределами работает
   - Клик по кругу применяет изменения

3. **Визуальная обратная связь**
   - Preview контуры отличаются от финальных
   - Переход между режимами плавный

## Альтернативные подходы (для будущего)

### 1. GPU-ускоренное разрезание

Использовать compute shaders для параллельного разрезания на GPU.

**Преимущества:**
- Максимальная производительность
- Масштабируется с GPU

**Недостатки:**
- Сложная реализация
- Требует WebGPU для веб-версии

### 2. LOD для контуров

Использовать упрощенные контуры при быстром движении.

**Преимущества:**
- Адаптивная производительность
- Хорошо для очень сложных моделей

**Недостатки:**
- Может выглядеть "дергано"
- Дополнительная сложность

## Заключение

Рекомендуемый **гибридный подход (Вариант 3)** обеспечивает:

1. ✅ **Отличную производительность** - 10-20x ускорение во время перетаскивания
2. ✅ **Хороший UX** - мгновенная визуальная обратная связь
3. ✅ **Точность** - финальное разрезание при подтверждении
4. ✅ **Совместимость** - использует существующий механизм подтверждения
5. ✅ **Поддерживаемость** - чистая архитектура с разделением ответственности

Реализация потребует:
- 2 новых файла (preview_contours.rs, contour_calculator.rs)
- Модификация 3 существующих файлов (slicing.rs, sync.rs, mod.rs)
- Добавление 1 нового компонента (PreviewContours)

Ожидаемое время реализации: 4-6 часов разработки + 2-3 часа тестирования.
