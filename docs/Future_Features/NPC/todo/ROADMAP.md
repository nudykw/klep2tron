# Дорожная карта: Actor Editor (NPC)

> [!IMPORTANT]
> Основной фокус: создание высокопроизводительных NPC моделей с возможностью кастомизации (Skins/VFX).

## Основные этапы

- [x] [01. Создание интерфейса](./01.ActorEditor.Interface.md)
- [x] [02. Импорт моделей](./02.ActorEditor.Import.md)
- [x] [03. Навигация во вьюпорте](./03.ActorEditor.Navigation.md)
- [x] [04. Система разрезания (Slicing)](./04.ActorEditor.Slicing.md)
- [x] [05. Работа с сокетами (Sockets)](./05.ActorEditor.Sockets.md)
- [x] [06. Сохранение и Baking](./06.ActorEditor.Save.md)
- [x] [07. Загрузка проекта](./07.ActorEditor.Load.md)
- [x] [08. Система Undo/Redo](./08.ActorEditor.UndoRedo.md)
- [x] [09. Горячие клавиши (Hotkeys)](./09.ActorEditor.Hotkeys.md)
- [x] [10. Режим инспекции (Inspection Mode)](./10.ActorEditor.Inspection.md)
- [ ] [11. Материалы деталей](./11.ActorEditor.Materials.md) (СЛЕДУЮЩИЙ ЭТАП)
- [x] [12. Оптимизация моделей (Integrated GPU focus)](./12.Model_Optimization.md)
- [x] [13. Масштабирование модели (Model Scaling)](./13.Model_Scaling.md)

---
> [!NOTE]
> Все реализованные задачи прошли проверку на производительность (Baking в .k2m) и стабильность (Undo/Redo, Conflict Resolve).

## Текущий статус
Редактор готов к созданию базовых NPC. Реализована полная цепочка: **Импорт -> Оптимизация -> Разрезание -> Настройка Сокетов -> Запекание в бинарные файлы для игры**.

**Следующая цель:** Реализация системы материалов и скинов (Задача 11).
