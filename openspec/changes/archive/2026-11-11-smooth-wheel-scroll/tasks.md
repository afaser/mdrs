## 1. Состояние прокрутки

- [x] 1.1 Создать `src/app/scroll.rs`: тип `Scroll` с полями `offset`, `limit`, `motion` и типом `Motion` — приватные поля, конструктор `new`; добавить `src/app/scroll.rs` в `src/app/mod.rs` приватным `mod scroll;` и реэкспортировать `Scroll` в блоке `pub use`; `cargo check` проходит.
- [x] 1.2 Реализовать в `scroll.rs` пересчёт `ScrollDelta` в смещение: восемь строк на щелчок, высота строки из общей константы в `src/app/view.rs`, для `Lines` — `y * 8 * line_height`, для `Pixels` — `y * 8 * line_height / 15.0`, знак инвертирован; тесты на оба вида дельты дают одинаковые 192 px и `cargo test` проходит.
- [x] 1.3 Реализовать `remember(viewport)` с обновлением `offset`/`limit`, отменой анимации при расхождении позиции окна и `motion.position` больше 0.5 px; тест на перетаскивание ползунка во время анимации подтверждает отмену, `cargo test` проходит.
- [x] 1.4 Реализовать `wheel(delta)`: создать или продолжить `Motion`, цель сложить с шагом и ограничить `0..=limit`; тесты на серию щелчков (итог равен сумме), верхнюю и нижнюю границы (движение не создаётся), направление вверх; `cargo test` проходит.
- [x] 1.5 Реализовать `frame(now)`: экспоненциальное сглаживание по прошедшему времени, снап при остатке 0.5 px и снятие `Motion`; тест подтверждает монотонность и завершение за 100–200 мс, `cargo test` проходит.

## 2. Перехват колеса

- [x] 2.1 Создать `src/app/wheel_scroll.rs` с `WheelScroll<'a>` вокруг `Element<'a, Message>`: ручная реализация `Widget` делегирует `size`, `layout`, `draw`, `tag`, `state`, `children`, `diff`, `operate`, `mouse_interaction`, `overlay` вложенному элементу, `cargo check` проходит.
- [x] 2.2 В `Widget::update` перехватывать вертикальный `WheelScrolled` без Shift над своим прямоугольником: публиковать `Message::Wheeled(delta)`, вызывать `shell.capture_event()` и не передавать событие ребёнку; чисто горизонтальную дельту и Shift+колесо пробрасывать ребёнку, `cargo check` проходит.
- [x] 2.3 Хранить модификаторы Shift в состоянии обёртки и обновлять их по `Event::Keyboard(Keyboard::ModifiersChanged)`; зарегистрировать модуль в `src/app/mod.rs` и реэкспортировать `WheelScroll`; `cargo check` проходит.
- [x] 2.4 Обернуть вертикальный `scrollable` документа в `src/app/view.rs` в `WheelScroll`, сохранив `SCROLL_ID`, направление, размеры и `on_scroll`; `cargo check` проходит.

## 3. Проводка сообщений и подписки

- [x] 3.1 Добавить `Message::Wheeled(mouse::ScrollDelta)` и `Message::ScrollFrame(Instant)` в `src/app/message.rs`; `cargo check` проходит.
- [x] 3.2 Заменить в `src/app/program.rs` хранение `AbsoluteOffset` на `Scroll`; методы `remember_scroll`, `scroll`, сброс при `open`, `reload` и новый `reset` делегировать в `Scroll`; `cargo check` проходит.
- [x] 3.3 Добавить в `Program` обработку колеса и кадра: `Wheeled` передаёт дельту в `Scroll::wheel`, `ScrollFrame` возвращает позицию из `Scroll::frame`; `moving` включает `iced::window::frames().map(Message::ScrollFrame)` в `Program::subscription`; `cargo check` проходит.
- [x] 3.4 В `src/app/update.rs` обработать новые сообщения: `Wheeled` и `ScrollFrame` возвращают `scroll_to` с текущей позицией, `Scrolled` обновляет `Scroll`; при `open` и `reload` отменять анимацию перед `scroll_to`; `cargo check` проходит.

## 4. Проверка поведения

- [x] 4.1 Запустить `cargo test`: тесты `scroll.rs` на шаг колеса, границы, серию щелчков, анимацию и отмену проходят.
- [x] 4.2 Запустить приложение на документе длиннее окна и проверить колесом: один щелчок сдвигает не менее восьми строк, движение плавное без рывка, серия щелчков складывается.
- [x] 4.3 Проверить границы: в начале и конце документа колесо не двигает содержимое за край и не оставляет бесконечной анимации.
- [x] 4.4 Проверить совместимость: перетаскивание ползунка во время анимации, выделение и копирование текста, live reload с сохранением позиции, горизонтальная прокрутка блока кода полосой и Shift+колесом.
- [x] 4.5 Запустить `cargo fmt --check`, `cargo check`, `cargo test`, `cargo clippy -- -D warnings`: все проходят без ошибок и предупреждений.
