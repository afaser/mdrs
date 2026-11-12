## Why

`AGENTS.md` — обязательная инструкция, но ревью всех 2928 строк `src` показывает расхождения почти в каждой подсистеме: файлы названы не по главному типу, два мега-файла, безымянные кортежи, мёртвые поля и сеттеры, дубли кэшей/отрисовки/конструкторов, if/else-выражения, bool-флаги, `f64`, боковые записи состояния, сравнения без допуска. Кроме стиля найдены дефекты: `font::isolate_system_fonts` пишет env из параллельных тестов, тест watcher недетерминирован; `WheelScroll::diff` уже реализован, но не покрыт регрессионным тестом. База до правок зелёная: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, 43 теста. Нужен рефакторинг без изменения наблюдаемого поведения.

## What Changes

- **Файлы и модули**
  - `document/source.rs` → `document/document.rs`; `selection/range.rs` → `selection/selection.rs`; `theme/highlight.rs` → `theme/highlighter.rs`.
  - Новые концепты: `selection/generation.rs` (`Generation`), `selection/{state,text_layout,decoration}.rs`, `app/{selection,motion,subscription}.rs`, `markdown/uri.rs`, `theme/typography.rs`.
  - Удаляется `open/argument.rs`; чтение аргумента — в `Program`, `SCROLL_ID` — в `view`.
  - `markdown/resolve.rs` разделяется на URI-кодек (`uri.rs`) и переписывание путей изображений (`resolve.rs`).
  - `selection/selectable_text.rs` разделяется на виджет-протокол, состояние, кэш раскладки и отрисовку украшений.
- **Сущности**
  - `SelectionRange { from, to }` вместо `Option<(Cursor, Cursor)>`; `Rewrite` — `from/to`; `Generation` вместо четырёх `u64`-эпох; `SelectionState` вместо пары `selection + selection_epoch`; `TextColors`, `TextConfig`, `TextLayout`, `TextState`, `LineRule`, `WheelState`.
  - `Drag { moved: bool }` → enum `Pressed | Selecting`; `Startup::enabled` → `Option<Startup>`; `Scroll`/`Motion` разделяются; `Gpu::select` становится чистой фазой.
  - Удаляются мёртвые поля/сеттеры `SelectableText` (`width`, `height`, `align_x`, `align_y`, `line_height`, `width()`), публичный `Highlighter::lines`, неиспользуемые `Clone`/`PartialEq`.
- **Поток и состояние**
  - Эпоха выделения пишется в `update` (граница), а не в `view`; `Viewer` теряет `Cell`; правило синхронизации: смена поколения очищает выделение, кроме активного drag.
  - `WheelScroll::diff` сохраняется и получает регрессионный тест; `WheelScroll::update` проверяет колесо через `Modifiers`, а не bool.
  - `Startup` существует только при `MDRS_STARTUP`, время — `f32`, выход процесса — в `update::first_frame`.
  - `Gpu::detect` — один проход (`inspect` + `fold`), выбор адаптера — чистая `select` с тестами.
  - `Highlighter` — один кэш `HashMap<String, HashMap<u64, Arc<[Span]>>>`, без аллокации языка на каждый кадр; `Highlighter::block` собирает spans за один проход.
  - `Scroll::wheel` принимает `now`; `Motion` получает операции; `resolve` возвращает `Cow<str>`; `isolate_system_fonts` идемпотентен через `Once`.
- **Тела и числа**
  - Убираются if/else-выражения, вложенные `if let … else`, bool-параметры; границы — `min/max`; абсолютность пути — `Path::join`; сравнения с нулём — именованные предикаты с `f32::EPSILON`; аккумуляторы — с явным типом; крупные замыкания — в именованные функции.
- **Тесты**
  - Негативные проверки «ничего не произошло» сливаются в позитивные сценарии; добавляются детерминированные тесты выбора GPU, `WheelScroll::diff`, не-UTF-8 файла, `file_path` для не-file URI, пустого и много-изображенческого `resolve`.
  - Ожидаемое число тестов — 40; тесты сценариев `markdown-viewer`/`text-selection` сохраняются.
- **BREAKING**: нет. Внешний API (`mdrs::run`) и наблюдаемое поведение не меняются.

## Capabilities

### New Capabilities

Нет. Это чистый рефакторинг: требования `markdown-viewer` и `text-selection` не меняются. В `.openspec.yaml` изменения стоит `skip_specs: true`; дельта-спецификация не создаётся.

### Modified Capabilities

Нет.

## Impact

- Затронуты все подсистемы `src`, кроме `main.rs`/`lib.rs`: `app`, `document`, `markdown`, `open`, `selection`, `theme`, `watch`.
- Новые файлы: `app/selection.rs`, `app/motion.rs`, `app/subscription.rs`, `markdown/uri.rs`, `selection/generation.rs`, `selection/state.rs`, `selection/text_layout.rs`, `selection/decoration.rs`, `theme/typography.rs`.
- Удаляется `open/argument.rs`; переименовываются `document/source.rs`, `selection/range.rs`, `theme/highlight.rs`.
- Новых зависимостей нет; правки `openspec/specs` нет.
- Проверка: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (40), ручной GUI-smoke и повторный проход алгоритма «Ревью».
