# 🛠 Level Editor Guide / Посібник редактора рівнів

[🇺🇸 English](#english) | [🇺🇦 Українська](#українська)

---

<a name="english"></a>
## 🇺🇸 English Guide

This guide describes how to use the **Klep2tron Level Editor** (`editor_client`).

### 🎮 Controls

#### 🎥 Camera Control
Hold **Shift** to use the camera controls:
- **Shift + Arrow Left / Right**: Rotate the camera around the center.
- **Shift + Arrow Up / Down**: Zoom in and out.
- **Shift + Q / A**: Raise or lower the camera height.

#### 📍 Selection
- **Arrow Keys**: Move the selection cursor on the 16x16 grid.
- **Mouse Left Click**: Directly select a tile in the 3D world.

#### 🏗 Editing Tiles
- **Q**: Increase tile height.
- **A**: Decrease tile height. Holes (`h = -1`) are allowed only for **Empty** tiles.
- **F**: **Clone Previous**. Copies the type and height from the cell you were at *just before* moving to the current one.
- **Top Panel**: Click on the 3D preview buttons at the top to change the active tile type.

#### 🏠 Room Management & Navigation
- **`[` / `]`**: Switch rooms with smooth transition.
- **F1 / Esc**: Toggle Global Help window.
- **Ctrl+Enter**: Toggle **Fullscreen** mode.
- **Esc**: Return to Main Menu (when help is closed).

### 💾 Saving
The editor **automatically saves** your progress to `map.json` in the project root whenever a change is detected.

---

<a name="українська"></a>
## 🇺🇦 Посібник (Українська)

Цей посібник описує, як користуватися **Редактором рівнів Klep2tron** (`editor_client`).

### 🎮 Керування

#### 🎥 Керування камерою
Утримуйте **Shift**, щоб керувати камерою:
- **Shift + Стрілки Вліво / Вправо**: Обертання камери навколо центру.
- **Shift + Стрілки Вгору / Вниз**: Наближення та віддалення (Zoom).
- **Shift + Q / A**: Зміна висоти камери.

#### 📍 Вибір (Selection)
- **Клавіші стрілок**: Переміщення курсора вибору по сітці 16x16.
- **Ліва кнопка миші**: Прямий вибір тайла у 3D світі.

#### 🏗 Редагування тайлів
- **Q**: Збільшити висоту тайла.
- **A**: Зменшити висоту тайла. Порожнечі (`h = -1`) дозволені лише для типу **Empty**.
- **F**: **Штамп**. Копіює тип та висоту з клітинки, на якій ви стояли *перед* цим.
- **Верхня панель**: Натискайте на кнопки з 3D-прев'ю вгорі, щоб змінити активний тип тайла.

#### 🏠 Керування кімнатами та Навігація
- **`[` / `]`**: Перемикання кімнат з плавним переходом.
- **F1 / Esc**: Відкрити/Закрити вікно допомоги.
- **Ctrl+Enter**: Повноэкранний режим.
- **Esc**: Вихід у головне меню (коли вікно допомоги закрите).

### 💾 Збереження
Редактор **автоматично зберігає** прогрес у файл `map.json` у корені проєкту при будь-якій зміні.
