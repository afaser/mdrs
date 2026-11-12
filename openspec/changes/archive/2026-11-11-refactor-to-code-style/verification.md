# Отчёт по изменению `refactor-to-code-style`

## База и итог

| Проверка | База | Итог |
|---|---|---|
| `cargo fmt --check` | 0 | 0 |
| `cargo clippy --all-targets -- -D warnings` | 0 | 0 |
| `cargo test` | 43 passed | 40 passed |

Итоговый набор тестов: `app::content` 4, `app::gpu` 2, `app::scroll` 8, `app::wheel_scroll` 2, `document::document` 3, `markdown::article` 2, `markdown::resolve` 6, `markdown::uri` 1, `open::link` 1, `selection::selection` 7, `theme::highlighter` 3, `watch::file_watcher` 1.

## Ручной GUI-smoke

Сессия niri, XWayland, окно 1256×1348, документ с заголовком, абзацем, ссылкой, изображением, таблицей, блоком кода и 40 абзацами.

- Заголовок, абзац, ссылка, изображение, таблица и блок кода отрисованы; подсветка кода цветная; изображение загружено из каталога документа.
- Выделение вперёд и назад подсвечено; Ctrl+C положил в буфер `aragraph with link and bol`; клик вне текста снял выделение.
- Колесо сдвинуло документ от начала к концу; промежуточные кадры анимации наблюдались.
- Live reload заменил заголовок на `Reloaded` и добавил абзацы; позиция чтения сохранилась.
- `MDRS_STARTUP=1 target/debug/mdrs /tmp/mdrs-smoke.md` напечатал `boot` и `first-frame`, завершился с кодом 0.
- `MDRS_LOG=1` напечатал два адаптера один раз и `selected wgpu adapter: AMD Radeon Graphics (RADV RENOIR)`.
- Перерисовка не падала; регрессионный тест `WheelScroll::diff` подтверждает сохранение дочернего дерева.
- Shift+колесо над блоком кода под синтетическим X11-событием не дало однозначного горизонтального сдвига; предикат `captures_wheel` покрыт тестом, само горизонтальное поведение оставлено без изменений и требует ручной проверки на реальном колесе.

## Покрытие сценариев

| Спецификация | Покрытие |
|---|---|
| `markdown-viewer` / открытие | аргумент и путь проверены smoke; `Content` 4 теста; диалог/перетаскивание не автоматизированы |
| `markdown-viewer` / рендер | smoke: заголовок, абзац, ссылка, изображение, таблица, код; `Article` 2 теста, `resolve` 6, `uri` 1, `highlighter` 3 |
| `markdown-viewer` / прокрутка | smoke колеса; `scroll` 8 тестов, `wheel_scroll` 2 теста |
| `markdown-viewer` / ссылки | `link` 1 тест; клик не автоматизирован |
| `markdown-viewer` / live reload | smoke; watcher 1 детерминированный тест |
| `markdown-viewer` / ошибки | `Content` missing/non-UTF-8 тесты |
| `markdown-viewer` / GPU | `gpu` 2 теста; `MDRS_LOG` smoke |
| `text-selection` / выделение | `selection` 7 тестов; smoke вперёд/назад, клик вне текста |
| `text-selection` / копирование | smoke Ctrl+C; `Selection::text` тесты |
| `text-selection` / ссылки | `TextState` жест сохраняет клик по ссылке; smoke клика не автоматизирован |

## Ревью по алгоритму AGENTS.md

Полный протокол находок и решений — в `design.md`. Ниже исполнение по изменённым файлам: темы файла, ответ по сущностям, дублям/состоянию, потоку и телу.

| Файл | Темы | Сущности | Дубли/состояние | Поток | Тело |
|---|---|---|---|---|---|
| `app/content.rs` | контент + тесты | `Content`; `Default::Empty` | нет | `open/reload -> ready -> article` | guard/match |
| `app/gpu.rs` | адаптеры + состояние проверки | `Gpu`, `GpuState`, `GpuCheck` | предикат `is_cpu` один | `detect -> select` | `select` чистая, `detect` один проход |
| `app/logging.rs` | логгер | `Stderr` | один `LEVEL` | нет | guard |
| `app/message.rs` | сообщения | enum `Message` | нет | нет | без логики |
| `app/motion.rs` | анимация | `Motion` | константы внутри | `start/retarget/advance/settled` | guard + `snap` |
| `app/program.rs` | состояние программы | `Program`, `SelectionState` | пара selection/epoch устранена | `update` синхронизирует поколение | getters, guard |
| `app/scroll.rs` | прокрутка | `Scroll` | `Motion` вынесен | `wheel(now)`, `frame` | `?`, `match`, предикат |
| `app/selection.rs` | состояние выделения | `SelectionState` | инвариант `clear` | `set/clear` | guard |
| `app/startup.rs` | замер старта | `Startup` | bool удалён | `new -> Option` | guard |
| `app/subscription.rs` | подписки и ввод | нет нового типа | один `match` по клавише | redraw постоянный | guard, `match` |
| `app/update.rs` | переходы сообщений | нет нового типа | `Picked` одна ветка | `first_frame` — граница | `match`, guard |
| `app/view.rs` | композиция | нет | `SCROLL_ID` у владельца | нет записи в `Viewer` | guard |
| `app/wheel_scroll.rs` | виджет колеса | `WheelScroll`, `WheelState` | предикат `is_zero` | `diff` делегирует детям | guard, `match` |
| `document/document.rs` | документ | `Document` | `Clone` удалён | `read_source -> Document` | guard |
| `document/mod.rs` | индекс | нет | нет | нет | только `mod`/`pub use` |
| `markdown/resolve.rs` | переписывание изображений | `Rewrite { from, to }` | `rewrite`/`apply_rewrites` | `image_rewrites -> apply` | guard |
| `markdown/uri.rs` | URI-кодек | нет нового типа | `split_first`, `write!` | `resolve_uri -> base.join` | guard |
| `markdown/mod.rs` | индекс | нет | нет | нет | только `mod`/`pub use` |
| `open/dialog.rs` | диалог | `Dialog` | `PartialEq` удалён | `subscription` | `matches!` |
| `open/link.rs` | ссылки | нет | список префиксов | `launch` | guard |
| `open/mod.rs` | индекс | нет | нет | нет | только `mod`/`pub use` |
| `selection/decoration.rs` | линии украшения | `LineRule` | одна функция линии | `draw` | `match` |
| `selection/generation.rs` | поколение | `Generation` | одно представление | `advance` | guard |
| `selection/selection.rs` | выделение | `Selection`, `SelectionRange` | `min/max`, `line_from/to` | `range -> text/bounds` | guard |
| `selection/selectable_text.rs` | виджет текста | `SelectableText`, `TextColors` | общий конструктор, `LineRule` | `layout/update` | `match`, guard |
| `selection/state.rs` | жест и состояние | `TextState`, `Drag`, `Release` | `sync_generation`, `reset_interaction` | `press/extend/release` | `match`, guard |
| `selection/text_layout.rs` | кэш раскладки | `TextConfig`, `TextLayout`, `LayoutChange` | один `Arc`, без `to_vec` | `sync -> rebuild/resize` | `match`, guard |
| `selection/mod.rs` | индекс | нет | нет | нет | только `mod`/`pub use` |
| `theme/font.rs` | шрифт | `Once` | идемпотентность | нет | guard |
| `theme/highlighter.rs` | подсветка | `Highlighter` | один вложенный кэш | `block -> highlight_block` | один проход |
| `theme/mod.rs` | индекс | нет | нет | нет | только `mod`/`pub use` |
| `theme/palette.rs` | палитра | константы | видимость `pub(super)` | нет | нет |
| `theme/style.rs` | стиль | нет | один `regular()` | `markdown_settings` | `match` |
| `theme/typography.rs` | размеры | константы | нет | нет | нет |
| `theme/viewer.rs` | viewer | `Viewer`, `TextRole` | один конструктор текста | `TextRole -> text` | `match` |
| `watch/file_watcher.rs` | watcher | `FileWatcher` | `touches` один предикат | `watch -> touches` | guard |

## Принятые исключения

- `selectable_text.rs` ~330 строк: один `impl Widget` (протокол, draw, update); чистые фазы вынесены, дальнейшее дробление потребовало бы `pub(super)`-полей на весь виджет.
- `Highlighter` использует `RefCell`-кэш в `&self` — memoization.
- `viewer::image` делает `is_file` на кадр — синхронный выбор alt-текста.
- `PathBuf`-копии при open/reload — независимые владения документом и watcher.
- `AbsoluteOffset.x` и `Motion.issued` сохранены — контракт iced и различие «выдано/наблюдено».
- `Article` остаётся концептом разбора с `items()`.
- `#[allow(clippy::module_inception)]` на `document/document.rs` и `selection/selection.rs` — имена файлов заданы картой рефакторинга.
