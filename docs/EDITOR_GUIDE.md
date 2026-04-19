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
- **Q**: Increase tile height. The tile will automatically take the currently selected type.
- **A**: Decrease tile height. If the height goes below 0, the tile becomes **Empty**.
- **F**: **Quick Clone**. Copies the tile type and height from a neighboring cell.
- **Top Panel**: Click on the 3D preview buttons at the top to change the active tile type (Cube, Wedge North/South/East/West).

#### 🏠 Room Management
- **`[` (Left Bracket)**: Switch to the previous room.
- **`]` (Right Bracket)**: Switch to the next room (creates a new room if you are at the end).

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
- **Q**: Збільшити висоту тайла. Тайл автоматично набуде поточно обраного типу.
- **A**: Зменшити висоту тайла. Якщо висота стане менше 0, тайл стане **Empty** (пустим).
- **F**: **Швидке клонування**. Копіює тип та висоту з сусіднього тайла.
- **Верхня панель**: Натискайте на кнопки з 3D-прев'ю вгорі, щоб змінити активний тип тайла (Куб, Скат).

#### 🏠 Керування кімнатами
- **`[` (Ліва дужка)**: Перейти до попередньої кімнати.
- **`]` (Права дужка)**: Перейти до наступної кімнати (створює нову, якщо ви в кінці списку).

### 💾 Збереження
Редактор **автоматично зберігає** прогрес у файл `map.json` у корені проєкту при будь-якій зміні.
