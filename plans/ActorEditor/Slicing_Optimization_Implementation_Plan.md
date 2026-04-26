# План реализации оптимизации слайсера (Вариант 3 - Гибридный подход)

> **Цель:** Ускорить отрисовку контуров при перетаскивании ползунка слайсера в 10-20 раз

## Обзор изменений

### Новые файлы (2)
1. `crates/client_core/src/actor_editor/geometry/contour_calculator.rs`
2. `crates/client_core/src/actor_editor/systems/preview_contours.rs`

### Модифицируемые файлы (3)
1. `crates/client_core/src/actor_editor/mod.rs`
2. `crates/client_core/src/actor_editor/systems/slicing.rs`
3. `crates/client_core/src/actor_editor/systems/sync.rs`

---

## Этап 1: Создание модуля быстрого вычисления контуров

### Файл: `crates/client_core/src/actor_editor/geometry/contour_calculator.rs`

**Задача:** Создать оптимизированный алгоритм вычисления только контуров (без создания мешей)

#### Шаг 1.1: Создать файл и базовую структуру

**Создать файл:** `crates/client_core/src/actor_editor/geometry/contour_calculator.rs`

```rust
//! Быстрое вычисление контуров разреза для preview режима
//!
//! Этот модуль предоставляет оптимизированные функции для вычисления только
//! контуров разреза без создания полных мешей. Используется для мгновенной
//! визуальной обратной связи во время перетаскивания ползунка слайсера.
//!
//! # Производительность
//!
//! - ~5-10ms для модели с 10K треугольников
//! - 10-20x быстрее полного разрезания
//! - Не создает новые меши и крышки
//!
//! # Использование
//!
//! ```rust
//! let segments = calculate_contours_only(&mesh, top_y, bottom_y);
//! ```

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, VertexAttributeValues};

/// Быстрое вычисление контуров разреза без создания мешей
///
/// Вычисляет только сегменты пересечения треугольников с плоскостями разреза.
/// НЕ создает новые треугольники, НЕ интерполирует нормали/UV/цвета, НЕ строит крышки.
///
/// # Аргументы
///
/// * `mesh` - Исходный меш для разрезания
/// * `top_y` - Y-координата верхней плоскости разреза (в локальных координатах меша)
/// * `bottom_y` - Y-координата нижней плоскости разреза (в локальных координатах меша)
///
/// # Возвращает
///
/// Вектор сегментов контуров `[start, end]` в локальных координатах меша
///
/// # Производительность
///
/// Для модели с 10K треугольников: ~5-10ms (vs ~50-200ms для полного разрезания)
pub fn calculate_contours_only(
    mesh: &Mesh,
    top_y: f32,
    bottom_y: f32,
) -> Vec<[Vec3; 2]> {
    // TODO: Реализация
    Vec::new()
}

/// Проверка пересечения треугольника с горизонтальной плоскостью
///
/// Упрощенная версия логики из `slicer::split_triangle()`, которая возвращает
/// только сегмент пересечения без создания новых треугольников.
///
/// # Аргументы
///
/// * `tri` - Массив из 3 вершин треугольника
/// * `plane_y` - Y-координата плоскости разреза
///
/// # Возвращает
///
/// `Some([p1, p2])` если треугольник пересекает плоскость, где p1 и p2 - точки пересечения
/// `None` если треугольник полностью выше или ниже плоскости
///
/// # Алгоритм
///
/// 1. Классифицировать вершины (выше/ниже плоскости)
/// 2. Если все вершины с одной стороны - нет пересечения
/// 3. Найти "одинокую" вершину (1 выше и 2 ниже, или наоборот)
/// 4. Вычислить точки пересечения ребер с плоскостью
/// 5. Вернуть сегмент между точками пересечения
fn intersect_triangle_with_plane(
    tri: &[Vec3; 3],
    plane_y: f32,
) -> Option<[Vec3; 2]> {
    // TODO: Реализация
    None
}
```

**Важно:** Обратите внимание на детальные комментарии - они помогут при отладке и будущей поддержке.

#### Шаг 1.2: Реализовать извлечение данных меша

```rust
pub fn calculate_contours_only(
    mesh: &Mesh,
    top_y: f32,
    bottom_y: f32,
) -> Vec<[Vec3; 2]> {
    // Извлечь позиции вершин
    let positions = if let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        p
    } else {
        return Vec::new();
    };
    
    // Извлечь индексы
    let indices = match mesh.indices() {
        Some(Indices::U16(i)) => i.iter().map(|&x| x as usize).collect::<Vec<usize>>(),
        Some(Indices::U32(i)) => i.iter().map(|&x| x as usize).collect::<Vec<usize>>(),
        None => (0..positions.len()).collect::<Vec<usize>>(),
    };
    
    let mut segments = Vec::new();
    
    // TODO: Обработка треугольников
    
    segments
}
```

#### Шаг 1.3: Реализовать обработку треугольников

```rust
pub fn calculate_contours_only(
    mesh: &Mesh,
    top_y: f32,
    bottom_y: f32,
) -> Vec<[Vec3; 2]> {
    // ... (код извлечения данных)
    
    let mut segments = Vec::new();
    
    // Обработка каждого треугольника
    for tri_indices in indices.chunks(3) {
        if tri_indices.len() < 3 { continue; }
        
        let tri = [
            Vec3::from(positions[tri_indices[0]]),
            Vec3::from(positions[tri_indices[1]]),
            Vec3::from(positions[tri_indices[2]]),
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
```

#### Шаг 1.4: Реализовать функцию пересечения

```rust
fn intersect_triangle_with_plane(
    tri: &[Vec3; 3],
    plane_y: f32,
) -> Option<[Vec3; 2]> {
    // Классификация вершин относительно плоскости
    let mut above_indices = Vec::new();
    let mut below_indices = Vec::new();
    
    for (i, vertex) in tri.iter().enumerate() {
        if vertex.y > plane_y {
            above_indices.push(i);
        } else {
            below_indices.push(i);
        }
    }
    
    // Треугольник не пересекает плоскость
    if above_indices.is_empty() || below_indices.is_empty() {
        return None;
    }
    
    // Определить "одинокую" вершину (1 с одной стороны, 2 с другой)
    let (lone_idx, other_indices) = if above_indices.len() == 1 {
        (above_indices[0], [below_indices[0], below_indices[1]])
    } else {
        (below_indices[0], [above_indices[0], above_indices[1]])
    };
    
    // Упорядочить индексы для правильного winding (как в slicer::split_triangle)
    // Это важно для корректной ориентации сегмента
    let (i1, i2) = if (lone_idx + 1) % 3 == other_indices[0] {
        (other_indices[0], other_indices[1])
    } else {
        (other_indices[1], other_indices[0])
    };
    
    let v_lone = tri[lone_idx];
    let v1 = tri[i1];
    let v2 = tri[i2];
    
    // Вычислить параметры интерполяции
    // ВАЖНО: Проверка деления на ноль для вырожденных случаев
    let dy1 = v1.y - v_lone.y;
    let dy2 = v2.y - v_lone.y;
    
    if dy1.abs() < 1e-6 || dy2.abs() < 1e-6 {
        // Вырожденный случай: ребро параллельно плоскости
        return None;
    }
    
    let t1 = (plane_y - v_lone.y) / dy1;
    let t2 = (plane_y - v_lone.y) / dy2;
    
    // Интерполяция позиций (только позиции, без нормалей/UV/цветов)
    let p1 = v_lone.lerp(v1, t1);
    let p2 = v_lone.lerp(v2, t2);
    
    Some([p1, p2])
}
```

**Добавлено:**
- Проверка деления на ноль для вырожденных случаев
- Комментарии о важности winding order
- Явное указание, что интерполируются только позиции

#### Шаг 1.5: Добавить экспорт в geometry/mod.rs

**Модифицировать файл:** `crates/client_core/src/actor_editor/geometry/mod.rs`

Добавить после существующих модулей:

```rust
pub mod slicer;
pub mod capper;
pub mod raycast;
pub mod contour_calculator; // НОВЫЙ МОДУЛЬ

use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct SlicedParts {
    pub head: Option<Mesh>,
    pub body: Option<Mesh>,
    pub legs: Option<Mesh>,
    pub contours: Vec<[Vec3; 2]>, // Segments for engraving and capping
}
```

**Проверка:** После добавления запустите `cargo check` для проверки компиляции модуля.

**Тестирование Этапа 1:**

Создать временный тест в конце файла `contour_calculator.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::PrimitiveTopology;
    use bevy::render::render_asset::RenderAssetUsages;
    
    #[test]
    fn test_intersect_simple_triangle() {
        // Треугольник: вершины на y=0, y=1, y=2
        let tri = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        ];
        
        // Плоскость на y=0.5 должна пересекать треугольник
        let result = intersect_triangle_with_plane(&tri, 0.5);
        assert!(result.is_some());
        
        if let Some([p1, p2]) = result {
            // Обе точки должны быть на y=0.5
            assert!((p1.y - 0.5).abs() < 1e-5);
            assert!((p2.y - 0.5).abs() < 1e-5);
        }
    }
    
    #[test]
    fn test_no_intersection_above() {
        // Треугольник полностью выше плоскости
        let tri = [
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(1.0, 3.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        ];
        
        let result = intersect_triangle_with_plane(&tri, 0.5);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_calculate_contours_cube() {
        // Создать простой куб
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        
        // Позиции вершин куба (упрощенно)
        let positions = vec![
            // Нижняя грань (y=0)
            [-0.5, 0.0, -0.5], [0.5, 0.0, -0.5], [0.5, 0.0, 0.5], [-0.5, 0.0, 0.5],
            // Верхняя грань (y=1)
            [-0.5, 1.0, -0.5], [0.5, 1.0, -0.5], [0.5, 1.0, 0.5], [-0.5, 1.0, 0.5],
        ];
        
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        
        // Индексы для боковых граней
        let indices = vec![
            0u32, 1, 5, 0, 5, 4, // Передняя грань
            1, 2, 6, 1, 6, 5,    // Правая грань
            2, 3, 7, 2, 7, 6,    // Задняя грань
            3, 0, 4, 3, 4, 7,    // Левая грань
        ];
        
        mesh.insert_indices(Indices::U32(indices));
        
        // Разрезать на y=0.5
        let segments = calculate_contours_only(&mesh, 0.5, 0.3);
        
        // Должны быть сегменты (4 боковые грани пересекаются)
        assert!(!segments.is_empty());
        println!("Found {} contour segments", segments.len());
    }
}
```

**Запуск тестов:**
```bash
cargo test --package client_core contour_calculator
```

**Ожидаемый результат:**
- ✅ Все тесты проходят
- ✅ Сегменты вычисляются корректно
- ✅ Нет паники на вырожденных случаях

**После успешного тестирования:** Можно удалить или закомментировать тесты, если они не нужны в production коде.

---

## Этап 2: Создание системы preview контуров

### Файл: `crates/client_core/src/actor_editor/systems/preview_contours.rs`

**Задача:** Создать систему, которая быстро вычисляет контуры во время перетаскивания

#### Шаг 2.1: Создать файл и базовую структуру

```rust
use bevy::prelude::*;
use super::super::{
    SlicingSettings, ActorBounds, OriginalMeshComponent, PreviewContours,
    geometry, EditorHelper,
};

pub fn preview_contours_system(
    mut commands: Commands,
    slicing_settings: Res<SlicingSettings>,
    actor_root_query: Query<(&ActorBounds, &GlobalTransform), With<crate::actor_editor::Actor3DRoot>>,
    mesh_query: Query<(Entity, &OriginalMeshComponent, &GlobalTransform)>,
    meshes: Res<Assets<Mesh>>,
    mut preview_query: Query<(Entity, &mut PreviewContours)>,
) {
    // TODO: Реализация
}
```

#### Шаг 2.2: Добавить проверку условий выполнения

```rust
pub fn preview_contours_system(
    mut commands: Commands,
    slicing_settings: Res<SlicingSettings>,
    actor_root_query: Query<(&ActorBounds, &GlobalTransform), With<crate::actor_editor::Actor3DRoot>>,
    mesh_query: Query<(Entity, &OriginalMeshComponent, &GlobalTransform)>,
    meshes: Res<Assets<Mesh>>,
    mut preview_query: Query<(Entity, &mut PreviewContours)>,
) {
    // Проверка 1: Система работает только во время перетаскивания
    if slicing_settings.dragging_gizmo.is_none() {
        // Удалить preview контуры если они есть
        for (entity, _) in preview_query.iter() {
            commands.entity(entity).remove::<PreviewContours>();
        }
        return;
    }
    
    // Проверка 2: Есть ли модель
    let Ok((bounds, root_global)) = actor_root_query.get_single() else {
        return;
    };
    
    // TODO: Вычисление контуров
}
```

#### Шаг 2.3: Реализовать вычисление контуров

```rust
pub fn preview_contours_system(
    // ... параметры
) {
    // ... проверки
    
    // Вычислить позиции плоскостей в локальных координатах
    let local_height = bounds.max.y - bounds.min.y;
    let plane_top_local = bounds.min.y + slicing_settings.top_cut * local_height;
    let plane_bottom_local = bounds.min.y + slicing_settings.bottom_cut * local_height;
    
    // Обработать каждый меш
    for (entity, original, transform) in mesh_query.iter() {
        if let Some(mesh) = meshes.get(&original.0) {
            // Преобразовать плоскости в локальные координаты меша
            let local_matrix = transform.compute_matrix();
            let inv_local = local_matrix.inverse();
            
            let world_top = root_global.compute_matrix().transform_point3(Vec3::new(0.0, plane_top_local, 0.0));
            let world_bottom = root_global.compute_matrix().transform_point3(Vec3::new(0.0, plane_bottom_local, 0.0));
            
            let mesh_local_top = inv_local.transform_point3(world_top).y;
            let mesh_local_bottom = inv_local.transform_point3(world_bottom).y;
            
            // Быстрое вычисление контуров
            let segments = geometry::contour_calculator::calculate_contours_only(
                mesh,
                mesh_local_top,
                mesh_local_bottom,
            );
            
            // Обновить или создать компонент PreviewContours
            if let Ok((_, mut preview)) = preview_query.get_mut(entity) {
                preview.segments = segments;
            } else {
                commands.entity(entity).insert(PreviewContours {
                    segments,
                    is_preview: true,
                });
            }
        }
    }
}
```

#### Шаг 2.4: Добавить экспорт в systems/mod.rs

```rust
// В файле crates/client_core/src/actor_editor/systems/mod.rs
pub mod preview_contours;
pub use preview_contours::preview_contours_system;
```

**Тестирование Этапа 2:**
- Запустить редактор с моделью
- Начать перетаскивать ползунок
- Проверить, что `PreviewContours` компонент создается
- Проверить, что сегменты обновляются каждый кадр

---

## Этап 3: Добавление компонента PreviewContours

### Файл: `crates/client_core/src/actor_editor/mod.rs`

**Задача:** Добавить новый компонент для хранения preview контуров

#### Шаг 3.1: Добавить компонент после SlicingContours

Найти определение `SlicingContours` (примерно строка 371) и добавить после него:

```rust
#[derive(Component)]
pub struct SlicingContours {
    pub segments: Vec<[Vec3; 2]>,
}

// НОВЫЙ КОМПОНЕНТ
#[derive(Component)]
pub struct PreviewContours {
    pub segments: Vec<[Vec3; 2]>,
    pub is_preview: bool,
}
```

#### Шаг 3.2: Добавить опциональный компонент для кеширования (будущая оптимизация)

```rust
// Опционально: для дополнительной оптимизации
#[derive(Component)]
pub struct CachedMeshPositions {
    pub positions: Vec<Vec3>,
    pub indices: Vec<usize>,
}
```

**Тестирование Этапа 3:**
- Проверить компиляцию проекта
- Убедиться, что компонент доступен в других модулях

---

## Этап 4: Модификация системы разрезания

### Файл: `crates/client_core/src/actor_editor/systems/slicing.rs`

**Задача:** Предотвратить полное разрезание во время перетаскивания

#### Шаг 4.1: Добавить проверку в начале mesh_slicing_system

Найти функцию `mesh_slicing_system` (примерно строка 11) и добавить проверку после обработки `pending_slices`:

```rust
pub fn mesh_slicing_system(
    // ... параметры
) {
    // 0. Handle Pre-sliced Meshes (from load)
    if !pending_slices.0.is_empty() {
        // ... существующий код
    }

    // НОВАЯ ПРОВЕРКА: Пропускаем разрезание во время перетаскивания
    // Preview контуры обрабатываются отдельной системой
    if slicing_settings.dragging_gizmo.is_some() && !slicing_settings.trigger_slice {
        return;
    }

    // 1. Check if a task is already running
    // ... остальной код без изменений
}
```

**Важно:** Эта проверка должна быть ПОСЛЕ обработки `pending_slices`, но ДО проверки `slicing_task`.

**Тестирование Этапа 4:**
- Запустить редактор
- Начать перетаскивать ползунок
- Проверить в логах, что `mesh_slicing_system` НЕ выполняется
- Отпустить мышь внутри круга
- Проверить, что `mesh_slicing_system` выполняется

---

## Этап 5: Модификация системы отрисовки контуров

### Файл: `crates/client_core/src/actor_editor/systems/sync.rs`

**Задача:** Обновить систему отрисовки для поддержки preview и final контуров

#### Шаг 5.1: Обновить сигнатуру функции

Найти функцию `draw_slicing_contours_system` (примерно строка 68) и обновить параметры:

```rust
pub fn draw_slicing_contours_system(
    // Добавить запрос для preview контуров
    preview_query: Query<(&PreviewContours, &GlobalTransform)>,
    // Переименовать существующий запрос для ясности
    final_query: Query<(&SlicingContours, &GlobalTransform)>,
    slicing_settings: Res<SlicingSettings>,
    viewport_settings: Res<ViewportSettings>,
    mut gizmos: Gizmos,
) {
    if !viewport_settings.slices { return; }
    
    // TODO: Обновить логику отрисовки
}
```

#### Шаг 5.2: Реализовать условную отрисовку

```rust
pub fn draw_slicing_contours_system(
    preview_query: Query<(&PreviewContours, &GlobalTransform)>,
    final_query: Query<(&SlicingContours, &GlobalTransform)>,
    slicing_settings: Res<SlicingSettings>,
    viewport_settings: Res<ViewportSettings>,
    mut gizmos: Gizmos,
) {
    if !viewport_settings.slices { return; }
    
    // Приоритет: preview контуры во время перетаскивания
    if slicing_settings.dragging_gizmo.is_some() {
        // Отрисовка preview контуров (оранжевый цвет)
        for (preview, transform) in preview_query.iter() {
            let matrix = transform.compute_matrix();
            for segment in &preview.segments {
                let start = matrix.transform_point3(segment[0]);
                let end = matrix.transform_point3(segment[1]);
                gizmos.line(start, end, Color::srgb(1.0, 0.5, 0.0)); // Оранжевый
            }
        }
    } else {
        // Отрисовка финальных контуров (красный цвет)
        for (contours, transform) in final_query.iter() {
            let matrix = transform.compute_matrix();
            for segment in &contours.segments {
                let start = matrix.transform_point3(segment[0]);
                let end = matrix.transform_point3(segment[1]);
                gizmos.line(start, end, Color::srgb(1.0, 0.0, 0.0)); // Красный
            }
        }
    }
}
```

**Тестирование Этапа 5:**
- Запустить редактор
- Проверить, что финальные контуры красные
- Начать перетаскивать ползунок
- Проверить, что контуры становятся оранжевыми
- Отпустить мышь и подтвердить
- Проверить, что контуры снова красные

---

## Этап 6: Регистрация систем

### Файл: `crates/client_core/src/actor_editor/mod.rs`

**Задача:** Добавить новую систему в расписание Bevy

#### Шаг 6.1: Найти секцию регистрации систем

Найти блок `.add_systems(Update, ...)` (примерно строка 155-170) и добавить новую систему:

```rust
.add_systems(Update, (
        ui::inspector::socket_ui_list_sync_system,
        ui::inspector::socket_ui_list_label_sync_system,
        // ... другие системы
        systems::scaling::mesh_scaling_apply_system,
        systems::preview_contours_system, // НОВАЯ СИСТЕМА
    ).run_if(in_state(GameState::ActorEditor)))
```

**Важно:** Система должна быть в `Update`, а не в `PostUpdate`, чтобы выполняться до `mesh_slicing_system`.

**Тестирование Этапа 6:**
- Проверить компиляцию
- Запустить редактор
- Проверить, что система выполняется (добавить временный `info!()` лог)

---

## Этап 7: Финальное тестирование и отладка

### Тест 1: Базовая функциональность

**Шаги:**
1. Загрузить модель в редактор
2. Начать перетаскивать верхний ползунок
3. Проверить:
   - ✅ Контуры обновляются плавно
   - ✅ Контуры оранжевые во время перетаскивания
   - ✅ FPS остается высоким (>60)
4. Отпустить мышь внутри круга
5. Проверить:
   - ✅ Выполняется полное разрезание
   - ✅ Контуры становятся красными
   - ✅ Меши обновляются корректно

### Тест 2: Механизм подтверждения

**Шаги:**
1. Начать перетаскивать ползунок
2. Отпустить мышь ЗА ПРЕДЕЛАМИ круга
3. Проверить:
   - ✅ Круг подтверждения остается видимым
   - ✅ Контуры остаются оранжевыми
   - ✅ Полное разрезание НЕ выполняется
4. Кликнуть по кругу
5. Проверить:
   - ✅ Выполняется полное разрезание
   - ✅ Контуры становятся красными

### Тест 3: Производительность

**Метрики для измерения:**
- FPS во время перетаскивания (ожидается >60)
- Время вычисления preview контуров (ожидается <10ms)
- Время полного разрезания (должно остаться без изменений)

**Инструменты:**
- Bevy diagnostic plugin для FPS
- `info!()` логи с `Instant::now()` для времени выполнения

### Тест 4: Различные модели

**Тестовые случаи:**
1. Простая модель (<1K треугольников) - должна работать мгновенно
2. Средняя модель (1K-10K треугольников) - должна быть плавной
3. Сложная модель (>10K треугольников) - должна быть приемлемой

### Тест 5: Граничные случаи

**Проверить:**
1. Быстрое движение ползунка туда-сюда
3. Переключение между моделями во время перетаскивания
4. Блокировка слайсера (locked) во время перетаскивания

---

## Этап 8: Оптимизации (опционально)

### Оптимизация 1: Кеширование позиций вершин

**Если производительность все еще недостаточна:**

```rust
// В preview_contours_system
// Вместо извлечения позиций каждый кадр, кешировать их
if let Some(cached) = cached_positions_query.get(entity) {
    // Использовать кешированные позиции
} else {
    // Извлечь и закешировать
}
```

### Оптимизация 2: Дебаунсинг

**Если обновления слишком частые:**

```rust
#[derive(Resource)]
pub struct PreviewDebounce {
    pub last_update: Instant,
    pub min_interval: Duration,
}

// В preview_contours_system
if debounce.last_update.elapsed() < debounce.min_interval {
    return; // Пропустить кадр
}
```

### Оптимизация 3: Spatial partitioning

**Для очень больших мешей:**

```rust
// Фильтровать треугольники, которые точно не пересекают плоскость
fn filter_relevant_triangles(
    triangles: &[Triangle],
    plane_y: f32,
    threshold: f32,
) -> Vec<usize> {
    // Возвращать только индексы треугольников рядом с плоскостью
}
```

---

## Чеклист реализации

### Этап 1: Модуль вычисления контуров
- [x] Создать файл `contour_calculator.rs`
- [x] Реализовать `calculate_contours_only()`
- [x] Реализовать `intersect_triangle_with_plane()`
- [x] Добавить экспорт в `geometry/mod.rs`
- [x] Протестировать на простом меше

### Этап 2: Система preview контуров
- [x] Создать файл `preview_contours.rs`
- [x] Реализовать `preview_contours_system()`
- [x] Добавить проверки условий выполнения
- [x] Добавить экспорт в `systems/mod.rs`
- [ ] Протестировать создание компонента

### Этап 3: Компонент PreviewContours
- [x] Добавить определение `PreviewContours` в `mod.rs`
- [x] Опционально: добавить `CachedMeshPositions`
- [x] Проверить компиляцию

### Этап 4: Модификация системы разрезания
- [x] Добавить проверку `dragging_gizmo` в `mesh_slicing_system`
- [x] Протестировать, что разрезание пропускается при перетаскивании
- [x] Протестировать, что разрезание выполняется при подтверждении

### Этап 5: Модификация отрисовки
- [x] Обновить сигнатуру `draw_slicing_contours_system`
- [x] Реализовать условную отрисовку preview/final
- [x] Протестировать смену цветов контуров
- [x] Настроить цветовую схему: черный для старой гравировки, оранжевый для preview, красный для финальной

### Этап 6: Регистрация систем
- [x] Добавить `preview_contours_system` в расписание
- [x] Проверить порядок выполнения систем
- [x] Протестировать, что система вызывается

### Этап 7: Тестирование
- [x] Тест базовой функциональности
- [x] Тест механизма подтверждения
- [x] Тест производительности
- [x] Тест различных моделей
- [x] Тест граничных случаев
- [x] Тест цветовой схемы (черный → оранжевый → красный)

### Этап 8: Оптимизации (если нужно) (вердикт - пока не нужно)
- [-] Кеширование позиций вершин
- [-] Дебаунсинг обновлений
- [-] Spatial partitioning

---

## Ожидаемые результаты

### Производительность
- **До оптимизации:** 4-14 FPS при перетаскивании (71-252ms на кадр)
- **После оптимизации:** 83-166 FPS при перетаскивании (6-12ms на кадр)
- **Улучшение:** 10-20x ускорение

### UX
- Мгновенная визуальная обратная связь при перетаскивании
- Плавное движение контуров без лагов
- Четкое визуальное различие между preview (оранжевый) и final (красный)
- Сохранение механизма подтверждения через круги

### Код
- Чистая архитектура с разделением ответственности
- Минимальные изменения существующего кода
- Хорошая расширяемость для будущих оптимизаций
- Понятная логика работы систем

---

## Возможные проблемы и решения

### Проблема 1: Контуры мерцают при быстром движении

**Причина:** Слишком частые обновления

**Решение:** Добавить дебаунсинг (Оптимизация 2)

### Проблема 2: Preview контуры не совпадают с финальными

**Причина:** Разная логика вычисления пересечений

**Решение:** Убедиться, что `intersect_triangle_with_plane()` использует ту же логику, что и `split_triangle()`

### Проблема 3: Утечка памяти при частых обновлениях

**Причина:** Создание новых векторов каждый кадр

**Решение:** Переиспользовать буферы или использовать `Vec::clear()` + `Vec::extend()`

### Проблема 4: Низкая производительность на сложных моделях

**Причина:** Обработка всех треугольников каждый кадр

**Решение:** Добавить spatial partitioning (Оптимизация 3)

### Проблема 5: Рассинхронизация с трансформациями

**Причина:** Неправильное преобразование координат

**Решение:** Убедиться, что используются правильные матрицы трансформации (root_global, local_matrix, inv_local)

---

## Дополнительные улучшения (для будущего)

### 1. Адаптивное качество
Автоматически снижать частоту обновлений при низком FPS

### 2. GPU-ускорение
Использовать compute shaders для вычисления контуров

### 3. LOD для контуров
Упрощать контуры при быстром движении

### 4. Предиктивное кеширование
Предвычислять контуры для нескольких позиций плоскостей

### 5. Визуальные эффекты
Добавить плавные переходы между preview и final режимами

---

## Заключение

Этот план обеспечивает пошаговую реализацию оптимизации слайсера с:
- Четкими