# Диаграммы архитектуры оптимизации слайсера

## 1. Текущая архитектура (До оптимизации)

```mermaid
graph TB
    subgraph "UI Layer"
        A[RangeSlider Widget]
        B[ConfirmationCircleUI]
    end
    
    subgraph "Update Schedule"
        C[range_slider_system]
    end
    
    subgraph "PostUpdate Schedule"
        D[mesh_slicing_system]
        E[draw_slicing_contours_system]
    end
    
    subgraph "Geometry Layer"
        F[split_mesh_by_planes]
        G[split_triangle]
        H[build_caps_from_segments]
    end
    
    subgraph "Resources"
        I[SlicingSettings]
        J[SlicingTask]
    end
    
    subgraph "Components"
        K[SlicingContours]
        L[ActorPart Meshes]
    end
    
    A -->|mouse drag| C
    C -->|update values| I
    I -->|values_changed| D
    D -->|async compute| J
    J -->|calls| F
    F -->|for each triangle| G
    F -->|if show_caps| H
    F -->|returns| K
    F -->|returns| L
    K -->|read| E
    E -->|Bevy Gizmos| M[GPU Rendering]
    
    style D fill:#ff9999
    style F fill:#ff9999
    style G fill:#ff9999
    style H fill:#ff9999
    
    classDef bottleneck fill:#ff9999,stroke:#ff0000,stroke-width:3px
```

### Проблемы текущей архитектуры

1. **Каждое движение ползунка** → Полное разрезание меша (50-200ms)
2. **Связанное выполнение** → Контуры ждут завершения разрезания
3. **Нет preview режима** → Пользователь не видит результат до завершения
4. **Механизм подтверждения не используется** → `needs_confirm` не предотвращает вычисления

---

## 2. Оптимизированная архитектура (Гибридный подход)

```mermaid
graph TB
    subgraph "UI Layer"
        A[RangeSlider Widget]
        B[ConfirmationCircleUI]
    end
    
    subgraph "Update Schedule"
        C[range_slider_system]
        D[preview_contours_system]
    end
    
    subgraph "PostUpdate Schedule"
        E[mesh_slicing_system]
        F[draw_slicing_contours_system]
    end
    
    subgraph "Geometry Layer - Fast Path"
        G[calculate_contours_only]
        H[intersect_triangle_with_plane]
    end
    
    subgraph "Geometry Layer - Full Path"
        I[split_mesh_by_planes]
        J[split_triangle]
        K[build_caps_from_segments]
    end
    
    subgraph "Resources"
        L[SlicingSettings]
        M[SlicingTask]
    end
    
    subgraph "Components"
        N[PreviewContours]
        O[SlicingContours]
        P[ActorPart Meshes]
    end
    
    A -->|mouse drag| C
    C -->|update values| L
    
    L -->|dragging_gizmo.is_some| D
    D -->|fast compute| G
    G -->|for each triangle| H
    G -->|returns segments| N
    
    L -->|trigger_slice = true| E
    E -->|async compute| M
    M -->|calls| I
    I -->|for each triangle| J
    I -->|if show_caps| K
    I -->|returns| O
    I -->|returns| P
    
    N -->|if dragging| F
    O -->|if confirmed| F
    F -->|Bevy Gizmos| Q[GPU Rendering]
    
    style D fill:#99ff99
    style G fill:#99ff99
    style H fill:#99ff99
    style N fill:#99ff99
    
    classDef optimized fill:#99ff99,stroke:#00ff00,stroke-width:3px
    classDef conditional fill:#ffff99,stroke:#ffaa00,stroke-width:2px
    
    class E conditional
```

### Преимущества оптимизированной архитектуры

1. **Preview режим** → Быстрые контуры (5-10ms) во время перетаскивания
2. **Разделение путей** → Fast path для preview, Full path для финала
3. **Условное выполнение** → Полное разрезание только при подтверждении
4. **Визуальная обратная связь** → Мгновенное обновление контуров

---

## 3. Диаграмма потока данных

### Сценарий 1: Перетаскивание ползунка (Preview Mode)

```mermaid
sequenceDiagram
    participant User
    participant RangeSlider
    participant SlicingSettings
    participant PreviewSystem
    participant ContoursCalculator
    participant DrawSystem
    participant GPU
    
    User->>RangeSlider: Начинает перетаскивание
    RangeSlider->>SlicingSettings: dragging_gizmo = Some(Top)
    
    loop Каждый кадр во время перетаскивания
        User->>RangeSlider: Движение мыши
        RangeSlider->>SlicingSettings: top_cut = new_value
        
        Note over PreviewSystem: Update Schedule
        PreviewSystem->>SlicingSettings: Проверка dragging_gizmo
        PreviewSystem->>ContoursCalculator: calculate_contours_only()
        
        Note over ContoursCalculator: ~5-10ms
        ContoursCalculator-->>PreviewSystem: Vec segments
        PreviewSystem->>PreviewContours: Обновление компонента
        
        Note over DrawSystem: PostUpdate Schedule
        DrawSystem->>SlicingContours: Чтение старых контуров
        DrawSystem->>GPU: Отрисовка черных линий (старая гравировка)
        DrawSystem->>PreviewContours: Чтение новых контуров
        DrawSystem->>GPU: Отрисовка оранжевых линий (preview)
        GPU-->>User: Визуальная обратная связь
    end
    
    Note over User: Видит мгновенное обновление контуров
```

### Сценарий 2: Подтверждение изменений (Full Slicing)

```mermaid
sequenceDiagram
    participant User
    participant RangeSlider
    participant SlicingSettings
    participant MeshSlicingSystem
    participant SlicingTask
    participant Slicer
    participant DrawSystem
    participant GPU
    
    User->>RangeSlider: Отпускает мышь внутри круга
    RangeSlider->>SlicingSettings: trigger_slice = true
    RangeSlider->>SlicingSettings: dragging_gizmo = None
    
    Note over MeshSlicingSystem: PostUpdate Schedule
    MeshSlicingSystem->>SlicingSettings: Проверка trigger_slice
    MeshSlicingSystem->>SlicingTask: Создание async task
    
    par Асинхронное выполнение
        SlicingTask->>Slicer: split_mesh_by_planes()
        Note over Slicer: ~50-200ms
        Slicer->>Slicer: Разрезание треугольников
        Slicer->>Slicer: Построение крышек
        Slicer-->>SlicingTask: SlicedParts + contours
    end
    
    SlicingTask-->>MeshSlicingSystem: Результат готов
    MeshSlicingSystem->>ActorPart: Создание новых мешей
    MeshSlicingSystem->>SlicingContours: Финальные контуры
    MeshSlicingSystem->>PreviewContours: Удаление preview
    
    DrawSystem->>SlicingContours: Чтение segments
    DrawSystem->>GPU: Отрисовка красных линий
    GPU-->>User: Финальный результат
```

### Сценарий 3: Отмена изменений (Release Outside)

```mermaid
sequenceDiagram
    participant User
    participant RangeSlider
    participant SlicingSettings
    participant PreviewSystem
    participant DrawSystem
    participant GPU
    
    User->>RangeSlider: Отпускает мышь за пределами круга
    RangeSlider->>SlicingSettings: needs_confirm = true
    Note over RangeSlider: dragging_gizmo остается Some()
    
    Note over PreviewSystem: Продолжает работать
    PreviewSystem->>PreviewContours: Контуры остаются
    
    DrawSystem->>PreviewContours: Отрисовка preview
    DrawSystem->>GPU: Оранжевые линии остаются
    
    Note over User: Видит круг подтверждения
    
    alt Пользователь кликает на круг
        User->>RangeSlider: Клик по кругу
        RangeSlider->>SlicingSettings: trigger_slice = true
        Note over MeshSlicingSystem: Выполняется полное разрезание
    else Пользователь кликает вне круга
        User->>RangeSlider: Клик вне круга
        RangeSlider->>SlicingSettings: Возврат к last_top/last_bottom
        RangeSlider->>PreviewContours: Удаление preview
        Note over DrawSystem: Возврат к старым контурам
    end
```

---

## 4. Сравнение производительности

```mermaid
graph LR
    subgraph "Текущая система"
        A1[Движение ползунка] -->|50-200ms| A2[Полное разрезание]
        A2 -->|20-50ms| A3[Построение крышек]
        A3 -->|1-2ms| A4[Отрисовка]
        A4 -->|71-252ms| A5[Результат]
    end
    
    subgraph "Оптимизированная система - Preview"
        B1[Движение ползунка] -->|5-10ms| B2[Вычисление контуров]
        B2 -->|1-2ms| B3[Отрисовка]
        B3 -->|6-12ms| B4[Результат]
    end
    
    subgraph "Оптимизированная система - Confirm"
        C1[Подтверждение] -->|50-200ms| C2[Полное разрезание]
        C2 -->|20-50ms| C3[Построение крышек]
        C3 -->|1-2ms| C4[Отрисовка]
        C4 -->|71-252ms| C5[Результат]
    end
    
    style A5 fill:#ff9999
    style B4 fill:#99ff99
    style C5 fill:#ffff99
```

### Метрики

| Операция | Текущая система | Оптимизированная (Preview) | Улучшение |
|----------|----------------|---------------------------|-----------|
| Движение ползунка | 71-252ms | 6-12ms | **10-20x** |
| FPS при перетаскивании | 4-14 FPS | 83-166 FPS | **20-40x** |
| Подтверждение | 71-252ms | 71-252ms | 1x (без изменений) |

---

## 5. Архитектура компонентов

```mermaid
classDiagram
    class SlicingSettings {
        +f32 top_cut
        +f32 bottom_cut
        +bool locked
        +Option~SlicingGizmoType~ dragging_gizmo
        +bool needs_confirm
        +bool trigger_slice
        +f32 last_top
        +f32 last_bottom
    }
    
    class PreviewContours {
        +Vec~Vec3_2~ segments
        +bool is_preview
    }
    
    class SlicingContours {
        +Vec~Vec3_2~ segments
    }
    
    class CachedMeshPositions {
        +Vec~Vec3~ positions
        +Vec~usize~ indices
    }
    
    class RangeSlider {
        +f32 min_value
        +f32 max_value
        +Option~RangeSliderThumb~ hovered_thumb
        +Option~RangeSliderThumb~ dragging
    }
    
    class ConfirmationCircleUI {
        +RangeSliderThumb thumb_type
    }
    
    SlicingSettings --> PreviewContours : controls
    SlicingSettings --> SlicingContours : controls
    RangeSlider --> SlicingSettings : updates
    ConfirmationCircleUI --> SlicingSettings : triggers
    CachedMeshPositions --> PreviewContours : used by
```

---

## 6. Диаграмма состояний механизма подтверждения

```mermaid
stateDiagram-v2
    [*] --> Idle: Модель загружена
    
    Idle --> Dragging: Начало перетаскивания
    
    state Dragging {
        [*] --> UpdatePreview
        UpdatePreview --> UpdatePreview: Движение мыши
        
        note right of UpdatePreview
            dragging_gizmo = Some(...)
            Preview контуры обновляются
            Полное разрезание НЕ выполняется
        end note
    }
    
    Dragging --> Confirmed: Отпускание внутри круга
    Dragging --> PendingConfirm: Отпускание вне круга
    
    state PendingConfirm {
        [*] --> ShowCircle
        
        note right of ShowCircle
            needs_confirm = true
            dragging_gizmo = Some(...)
            Круг подтверждения виден
            Preview контуры остаются
        end note
    }
    
    PendingConfirm --> Confirmed: Клик по кругу
    PendingConfirm --> Cancelled: Клик вне круга
    
    state Confirmed {
        [*] --> ExecuteSlicing
        
        note right of ExecuteSlicing
            trigger_slice = true
            Полное разрезание выполняется
            Preview контуры удаляются
        end note
    }
    
    Confirmed --> Idle: Разрезание завершено
    Cancelled --> Idle: Возврат к старым значениям
```

---

## 7. Диаграмма файловой структуры

```
crates/client_core/src/actor_editor/
│
├── mod.rs
│   ├── [ИЗМЕНЕНИЕ] Добавить PreviewContours компонент
│   ├── [ИЗМЕНЕНИЕ] Добавить CachedMeshPositions компонент
│   └── [ИЗМЕНЕНИЕ] Обновить порядок систем
│
├── widgets/
│   └── sliders.rs
│       └── [БЕЗ ИЗМЕНЕНИЙ] Механизм подтверждения уже работает
│
├── systems/
│   ├── slicing.rs
│   │   └── [ИЗМЕНЕНИЕ] Добавить проверку dragging_gizmo
│   │
│   ├── sync.rs
│   │   └── [ИЗМЕНЕНИЕ] Обновить draw_slicing_contours_system
│   │
│   └── preview_contours.rs
│       └── [НОВЫЙ] Система быстрого вычисления контуров
│
└── geometry/
    ├── slicer.rs
    │   └── [БЕЗ ИЗМЕНЕНИЙ] Полное разрезание
    │
    ├── capper.rs
    │   └── [БЕЗ ИЗМЕНЕНИЙ] Построение крышек
    │
    └── contour_calculator.rs
        └── [НОВЫЙ] Быстрое вычисление только контуров
```

---

## 8. Диаграмма взаимодействия систем

```mermaid
graph TB
    subgraph "Input Layer"
        A[Mouse Input]
    end
    
    subgraph "UI Systems Update"
        B[range_slider_system]
        C[slicer_lock_system]
    end
    
    subgraph "Preview Systems Update"
        D[preview_contours_system]
        E{dragging_gizmo?}
    end
    
    subgraph "Slicing Systems PostUpdate"
        F[mesh_slicing_system]
        G{trigger_slice?}
        H[AsyncComputeTaskPool]
    end
    
    subgraph "Rendering Systems PostUpdate"
        I[draw_slicing_contours_system]
        J{Какие контуры?}
        K[Bevy Gizmos]
    end
    
    A --> B
    B --> C
    C --> D
    
    D --> E
    E -->|Yes| L[calculate_contours_only]
    E -->|No| M[Skip]
    L --> N[PreviewContours]
    
    C --> F
    F --> G
    G -->|Yes| H
    G -->|No| O[Skip]
    H --> P[split_mesh_by_planes]
    P --> Q[SlicingContours]
    P --> R[ActorPart Meshes]
    
    N --> I
    Q --> I
    I --> J
    J -->|Preview| S[Orange Lines]
    J -->|Final| T[Red Lines]
    S --> K
    T --> K
    
    style D fill:#99ff99
    style L fill:#99ff99
    style N fill:#99ff99
    style E fill:#ffff99
    style G fill:#ffff99
    style J fill:#ffff99
```

---

## 9. Временная диаграмма выполнения

```mermaid
gantt
    title Сравнение времени выполнения операций
    dateFormat X
    axisFormat %L ms
    
    section Текущая система
    Движение ползунка 1    :a1, 0, 71ms
    Движение ползунка 2    :a2, after a1, 71ms
    Движение ползунка 3    :a3, after a2, 71ms
    Подтверждение          :a4, after a3, 71ms
    
    section Оптимизированная
    Движение ползунка 1    :b1, 0, 6ms
    Движение ползунка 2    :b2, after b1, 6ms
    Движение ползунка 3    :b3, after b2, 6ms
    Подтверждение          :b4, after b3, 71ms
```

**Итого:**
- Текущая: 4 операции × 71ms = **284ms**
- Оптимизированная: 3 × 6ms + 1 × 71ms = **89ms**
- **Экономия: 195ms (68%)**

---

## 10. Диаграмма принятия решений

```mermaid
flowchart TD
    Start([Пользователь двигает ползунок]) --> CheckLocked{Слайсер заблокирован?}
    
    CheckLocked -->|Да| End1([Ничего не делать])
    CheckLocked -->|Нет| UpdateUI[Обновить UI значения]
    
    UpdateUI --> CheckDragging{Идет перетаскивание?}
    
    CheckDragging -->|Да| FastPath[Preview Path]
    CheckDragging -->|Нет| CheckTrigger{trigger_slice?}
    
    FastPath --> CalcContours[calculate_contours_only]
    CalcContours --> UpdatePreview[Обновить PreviewContours]
    UpdatePreview --> DrawPreview[Отрисовать оранжевые линии]
    DrawPreview --> End2([Готово - 6-12ms])
    
    CheckTrigger -->|Нет| End3([Ничего не делать])
    CheckTrigger -->|Да| FullPath[Full Slicing Path]
    
    FullPath --> AsyncTask[Создать async task]
    AsyncTask --> SplitMesh[split_mesh_by_planes]
    SplitMesh --> BuildCaps{show_caps?}
    
    BuildCaps -->|Да| AddCaps[Построить крышки]
    BuildCaps -->|Нет| SkipCaps[Пропустить]
    
    AddCaps --> CreateMeshes[Создать меши частей]
    SkipCaps --> CreateMeshes
    
    CreateMeshes --> UpdateFinal[Обновить SlicingContours]
    UpdateFinal --> RemovePreview[Удалить PreviewContours]
    RemovePreview --> DrawFinal[Отрисовать красные линии]
    DrawFinal --> End4([Готово - 71-252ms])
    
    style FastPath fill:#99ff99
    style CalcContours fill:#99ff99
    style UpdatePreview fill:#99ff99
    style DrawPreview fill:#99ff99
    style End2 fill:#99ff99
    
    style FullPath fill:#ffff99
    style AsyncTask fill:#ffff99
    style End4 fill:#ffff99
```

---

## Заключение

Диаграммы демонстрируют:

1. **Четкое разделение** между preview и full slicing путями
2. **Условное выполнение** тяжелых операций только при необходимости
3. **Асинхронность** для предотвращения блокировки UI
4. **Визуальную обратную связь** через разные цвета контуров
5. **Значительное улучшение производительности** (10-20x) во время перетаскивания

Архитектура спроектирована для:
- Минимальных изменений существующего кода
- Максимальной производительности в интерактивном режиме
- Сохранения точности при финальном разрезании
- Хорошей расширяемости для будущих оптимизаций
