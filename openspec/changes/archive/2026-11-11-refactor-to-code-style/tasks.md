> CPU-bound команды (`cargo check`, `cargo test`, `cargo clippy`, `cargo run`) запускаются с минимальным приоритетом — `nice -n 19 <command>`; число потоков не ограничивается.

## 1. Исходная точка

- [x] 1.1 Зафиксировать базу: `nice -n 19 cargo fmt --check`, `nice -n 19 cargo clippy --all-targets -- -D warnings`, `nice -n 19 cargo test` — зелёные, 43 теста; числа записать в отчёт изменения.

## 2. Файлы и модули

- [x] 2.1 Переименовать `src/document/source.rs` → `src/document/document.rs`; вынести чтение файла в именованный `read_source`; удалить неиспользуемый `Clone` у `Document`; обновить `document/mod.rs`; проверка: `cargo check`, `cargo test document` — 3 теста.
- [x] 2.2 Переименовать `src/selection/range.rs` → `src/selection/selection.rs`; ввести приватный `SelectionRange { from, to }` с конструктором, нормализующим порядок и отсекающим равные курсоры; перевести `Selection::range/text/bounds` на него, убрать `Option<(Cursor, Cursor)>`, if/else-выражения, `if text.is_empty()` и неиспользуемый `PartialEq`; проверка: `cargo test selection` — 7 тестов (после слияния 6.4), `cargo check` без предупреждений.
- [x] 2.3 Переименовать `src/theme/highlight.rs` → `src/theme/highlighter.rs`; обновить `theme/mod.rs`; проверка: `cargo check`, `cargo test theme::highlighter` — 3 теста (после 6.5).
- [x] 2.4 Разделить `src/markdown/resolve.rs`: `src/markdown/uri.rs` (percent-кодек, `has_scheme`, `file_path`, `resolve_uri`) и `src/markdown/resolve.rs` (`resolve`, `Rewrite { from, to, replacement }`, `image_rewrites`, `destination_range`, `apply_rewrites`); обновить `markdown/mod.rs`; проверка: `cargo test markdown` — 7 тестов проходят без ослабления ассертов.
- [x] 2.5 Разделить `src/selection/selectable_text.rs`: `text_layout.rs` (`TextConfig`, `TextLayout`, `LayoutChange`, общий `text(content, bounds, config)`), `state.rs` (`TextState`, enum `Drag`, поколение, методы жеста), `decoration.rs` (enum `LineRule`, одна функция линии); `SelectableText` получает `TextColors` в конструкторе, мёртвые поля/сеттеры `width/height/align_*/line_height` удаляются; `TextLayout` хранит `Arc<[Span]>` без `to_vec`, `reset_interaction` сбрасывает и `hovered_span`; проверка: `cargo check`, `cargo test selection` — 7 тестов (после 6.4).
- [x] 2.6 Добавить `src/selection/generation.rs` с `Generation` (`new`, `advance`, сравнение) и реэкспортом из `selection`; перевести поля эпох на тип без изменения поведения; проверка: `cargo check` без предупреждений.
- [x] 2.7 Добавить `src/app/selection.rs` (`SelectionState`), `src/app/motion.rs` (`Motion`), `src/app/subscription.rs` (`subscription` + маппинг событий), `src/theme/typography.rs` (`TEXT_SIZE`, `TEXT_LINE_HEIGHT`, `CODE_SIZE`); обновить индексы `app/mod.rs` и `theme/mod.rs`; проверка: `cargo check`.
- [x] 2.8 Удалить `src/open/argument.rs`, перенести чтение аргумента в `app/program.rs`; перенести `SCROLL_ID` в `app/view.rs` (`pub(super)`); обновить `open/mod.rs` и импорты; проверка: `cargo check`, запуск с путём в аргументах — в 7.2.
- [x] 2.9 Перенести типографику и `markdown_settings` в `theme` (`typography.rs`, `style.rs`), стилевые `code_style`/`heading_size`/`heading_top` — в `style.rs`; `app/view.rs` оставить композицией, `app/scroll.rs` импортирует `theme::TEXT_LINE_HEIGHT`; проверка: `cargo check`, в `scroll.rs` нет импорта `crate::app::view`.

## 3. Сущности, поток и состояние

- [x] 3.1 Эпоха: `Viewer` хранит `Generation` (без `Cell`), `set_generation(&mut self)`; `Program` синхронизирует `Viewer` в `update` после обработки сообщения; `TextState::sync_generation` очищает выделение при смене поколения, кроме активного drag; убрать ручной `wrapping_add(1)`; проверка: `cargo check`, матрица выделения в 7.2.
- [x] 3.2 `SelectionState`: `set(Some)` меняет текст, `clear()` обнуляет и продвигает `Generation`; `Program` хранит `SelectionState`, `reload` использует `mem::take` (`Content` получает `Default::Empty`), sentinel-`mem::replace` удаляется; проверка: `cargo check`, тесты `Content` проходят.
- [x] 3.3 `Startup`: `new(started) -> Option<Self>` через предикат `measurement_requested()`, без bool-флага и двойных guard; `elapsed_ms -> f32`; `finish(self) -> f32` без `std::process::exit`; выход — в `update::first_frame`; подписка на `RedrawRequested` остаётся постоянной, иначе GPU-check не запускается без `MDRS_STARTUP`; проверка: `cargo check`, `MDRS_STARTUP=1 cargo run -- README.md` печатает замеры и завершается после первого кадра, без переменной — приложение живёт и логирует адаптеры.
- [x] 3.4 `Gpu`: чистая `select` над итератором `Gpu` (первый `Hardware`, иначе первый, иначе `Software("no Vulkan adapter")`); `detect` — `map` → `inspect` → `fold` за один проход; `GpuState::apply` без логирования; `begin_check` возвращает enum вместо bool; проверка: `cargo check`, `MDRS_LOG=1 cargo run -- README.md` логирует адаптеры один раз и выбирает аппаратный.
- [x] 3.5 `Highlighter`: один кэш `HashMap<String, HashMap<u64, Arc<[Span]>>>`, lookup по `&str` без аллокации языка; `block` собирает spans за один проход; `lines`, `Line`, `Key` удаляются; `Highlighter` — `pub(super)`; проверка: `cargo test theme::highlighter` — тесты переписаны на `block`, 3 теста (после 6.5).
- [x] 3.6 `Viewer`: один конструктор `SelectableText` с `TextRole { Heading, Paragraph, Code }` и `TextColors`; `Generation` вместо `Cell`; проверка: `cargo check`, визуальные роли — в 7.2.
- [x] 3.7 `Scroll`/`Motion`: вынести `Motion` с `start/retarget/advance/settled`; `wheel(delta, now)` получает время; `frame` — guard и `snapped`; `Scroll` реализует `Default`, `reset` без дубля конструктора; `issued` и `AbsoluteOffset.x` сохраняются; проверка: `cargo test scroll` — 8 тестов проходят (после 6.3).
- [x] 3.8 `WheelScroll`: сохранить `diff` через `tree.diff_children`; `State` → `WheelState`; `captures_wheel` принимает `Modifiers`; нулевая дельта — предикат с `f32::EPSILON`; проверка: `cargo test wheel_scroll` — тесты проходят, новый тест `diff` — в 6.6.
- [x] 3.9 `font::isolate_system_fonts`: обернуть в `Once`, убрать повторную запись env из параллельных тестов; проверка: `cargo test selection` под параллельным запуском без предупреждений/гонок.
- [x] 3.10 `resolve`/`uri`: `resolve -> Cow<str>`; `decode_percent` через `split_first` без индексов; `encode_percent` через `write!` без `format!` на байт; `resolve_uri` — `base.join`; крупные замыкания — `rewrite`/`run_rect`; проверка: `cargo test markdown` — 7 тестов.
- [x] 3.11 `update`/`program`: `Picked` — одна ветка без дубля `finish_dialog`; убрать `stop_scroll` из `open_document`; `wheel` получает `Instant::now()` на границе; `Scroll`-позиция превращается в `AbsoluteOffset` в `update`; `Program::subscription` делегирует в `subscription`; проверка: `cargo check`, `cargo test app` — тесты `Content` проходят.
- [x] 3.12 `logging`: один уровень логирования (`const LEVEL: Level = Level::Info`, `LEVEL.to_level_filter()`) вместо двух объявлений; проверка: `cargo check`, `MDRS_LOG=1` печатает строки один раз.

## 4. Тела, дубли и числа

- [x] 4.1 Убрать if/else-выражения: `Scroll::frame`, `resolve_uri`, `Selection::text`, `viewer::heading`, `encode_percent`, `Selection::range` — guard-выход/именованные функции/`min/max`; проверка: `cargo test` по затронутым модулям.
- [x] 4.2 Ввести предикаты с `f32::EPSILON` для сравнений с нулём: `Scroll::wheel`, `WheelScroll`, `Selection::bounds`, `cursor_at`; намеренные допуски анимации не трогать; проверка: `cargo test scroll selection wheel_scroll`.
- [x] 4.3 Дубли: ярлыки Ctrl+O/Ctrl+C — один `match`; `link` — список префиксов; `style` — один `regular()`; `view::settings` — struct update без `let mut`; `update::Picked` — одна ветка; проверка: `cargo check`, `cargo test open`.
- [x] 4.4 Мёртвое/имена: удалить `let _ = renderer`, неиспользуемые поля/сеттеры/derive (`Document::Clone`, `Selection::PartialEq`, `Dialog::PartialEq`); `Dialog` — `matches!` вместо `==`; `hovered` → `hovered_span`, `Drag.link` → `link_span`; `cursor_at` — именованный зазор вместо `0.01`; общий helper относительной точки вместо `inside`+`position`; `GpuState` без логов; проверка: `cargo clippy --all-targets -- -D warnings`.

## 5. Видимость и индексы

- [x] 5.1 Политика видимости: `pub` + `pub use` — API подсистемы, внутренние межфайловые элементы — `pub(super)` (`Highlighter`, `palette` кроме `DANGER`, `SCROLL_ID`, `gpu::{detect,select,GpuState,GpuCheck}`, `subscription`, `Startup`, `logging::init`, `SelectionState`, `Motion`, `TextState`, `TextConfig`, `TextLayout`, `LineRule`); реэкспорт `Gpu`, `Generation`, `TextColors`, `markdown_settings`, размеров типографики; проверка: `cargo clippy --all-targets -- -D warnings` без `private_interfaces`.
- [x] 5.2 Индексы: приватные `mod` и `pub use` в алфавитном порядке, нет ссылок на `open::path`, `source`, `range`, `highlight`; проверка: `cargo check` без предупреждений.

## 6. Тесты

- [x] 6.1 `resolve`: слить external/data/anchor в позитивный relative-тест; добавить пустой источник и несколько изображений; проверка: `cargo test markdown::resolve` — 6 тестов.
- [x] 6.2 `watch`: заменить недетерминированный integration-тест на детерминированный `touches`-сценарий (позитив + отрицательные unrelated/access); реальный watcher — ручной smoke; проверка: `cargo test watch` — 1 тест.
- [x] 6.3 `scroll`: слить external/matching в один сценарий; проверка: `cargo test scroll` — 8 тестов.
- [x] 6.4 `selection`: слить два bounds-теста в один сценарий визуальных строк; проверка: `cargo test selection` — 7 тестов.
- [x] 6.5 `highlighter`: удалить `repeated_lookup_returns_identical_cache`; проверка: `cargo test theme::highlighter` — 3 теста.
- [x] 6.6 `wheel_scroll`: слить три предиката в один сценарий; добавить `diff`-тест на сохранение дочернего дерева; проверка: `cargo test wheel_scroll` — 2 теста.
- [x] 6.7 Добавить `gpu::select` (приоритет Hardware; пустой список), не-UTF-8 файл в `Content`, `file_path` для не-file URI; проверка: `cargo test gpu`, `cargo test content`, `cargo test uri` — по 2/4/1 тесту.
- [x] 6.8 Сверить оставшиеся тесты со сценариями `markdown-viewer` и `text-selection`: каждый сценарий покрыт тестом или ручной проверкой 7.2; список записать в отчёт изменения.
- [x] 6.9 Итог: `cargo test` — 40 тестов (43 − 10 слияний/удалений + 7 добавлений).

## 7. Проверки

- [x] 7.1 Финальный прогон: `nice -n 19 cargo fmt --check`, `nice -n 19 cargo clippy --all-targets -- -D warnings`, `nice -n 19 cargo test` — зелёные, 40 тестов.
- [x] 7.2 Ручной GUI-smoke: заголовки/абзацы/ссылка/изображение/таблица/код; выделение вперёд/назад, клик вне текста, Ctrl+C; ссылка (unit `link`, клик не автоматизирован); колесо дискретной дельтой (smoke) и пиксельной (unit); Shift+колесо (предикат unit, ручной результат неоднозначен); live reload с сохранением позиции; запуск с путём; `MDRS_STARTUP=1`; перерисовка (проверка `WheelScroll::diff`); расхождения записаны в `verification.md`.
- [x] 7.3 Пройти алгоритм «Ревью» по каждому изменённому файлу: выписать темы, ответы по сущностям, дублям, потоку и телу; «находок нет» — только с ответами; результат записать в отчёт изменения.
