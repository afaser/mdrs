# Design: own-markdown-engine

## Context

См. `proposal.md` — Why. Текущее состояние, важное для подхода:

- `Cargo.toml` содержит `[patch.crates-io]` на три копии iced в `vendor/`. Реальные отличия от crates.io — пять файлов:
  - `iced_core/src/text.rs` — поле `Span.letter_spacing`;
  - `iced_graphics/src/text/paragraph.rs` — применение spacing через cosmic-text `Attrs`;
  - `iced_widget/src/markdown.rs` — source-диапазоны, `Text::source_segments`, `Row::cells`, диапазоны code-строк;
  - `iced_widget/src/column.rs`, `row.rs` — culling `CursorMoved`/`mouse_interaction`;
  - `iced_widget/src/scrollable.rs` — публичный `State` с `absolute_offset`/`set_offset` и пропуск `RedrawRequested` в content.
- `AGENTS.md` запрещает `vendor/` и `[patch.crates-io]`, разрешает только crates.io, требует GPU-рендер и проверку через sandbox.
- Приложение уже владеет markdown-политикой: `Article`, `SourceIndex`, `DocumentSelection`, `DocumentGeometry`, `SelectableText`, `InlineFlow`, `SpanStyler`. Вендоренный `iced_widget::markdown` используется только как парсер в `Item`/`Text` и как сборка блоков.
- Iced-виджеты (`column`, `row`, `container`, `scrollable`, `table`, `image`, `svg`, `rule`, `text`, `checkbox`, `mouse_area`, `space`) остаются примитивами раскладки и отрисовки. См. обсуждение границы в change.

## Goals / Non-Goals

**Goals:**

- Удалить `vendor/` и `[patch.crates-io]` без потери поведения.
- Перенести в проект: межбуквенный интервал, source-map markdown, culling, доступ к offset прокрутки.
- Сохранить текущую геометрию и ресайз: ширину текста, переносы, inline-чипы, позицию прокрутки, выделение и копирование.
- Сохранить производительность колеса: анимация остаётся в `WheelScroll`, без сообщений приложения на кадр.
- Оставить iced-виджеты примитивами; не переписывать `scrollable`/`table`/`column`/`row`.

**Non-Goals:**

- Свой handwritten GFM-парсер вместо `pulldown-cmark`.
- Свои `column`/`row`/`scrollable`/`table`/`image`/`svg`.
- Новые возможности UI, новые поля конфига, изменение формата markdown.
- Исправление существующего ограничения «чип шире ячейки/строки обрезается».
- Исправление x-координат выделения внутри горизонтально прокрученной таблицы.
- Инкрементальный `Content`, broken-link callback и прочие streaming-возможности вендоренного markdown, которые приложение не использует.

## Decisions

### 1. Свой текстовый слой на cosmic-text через публичный API iced

**Решение:** новый `src/text/`:

- `Span` — собственный value-тип с полями `text`, `size`, `line_height`, `letter_spacing`, `font`, `color`, `link`, `highlight`, `padding`, `underline`, `strikethrough`;
- `Paragraph` — собственная обёртка над `cosmic_text::Buffer`, с `resize`, `compare`, `min_bounds`, `span_bounds`, `hit_span`, `buffer`, `draw`;
- `Label` — неинтерактивный виджет для экрана сообщения и alt-текста.

Ключевой факт: в iced 0.14 публичны `iced::advanced::graphics::text::{font_system, to_attributes, align, Raw, Renderer as TextRenderer}`. `Raw.buffer` — `Weak<cosmic_text::Buffer>`, а `fill_raw` реализован у `iced::Renderer` (iced_wgpu и fallback). Поэтому можно строить свой buffer и рисовать его без форка:

```rust
let mut fs = lock_font_system();        // без panic на poisoned lock
let mut buffer = Buffer::new(fs.raw(), Metrics::new(size, line_height));
buffer.set_size(fs.raw(), Some(bounds.width), Some(bounds.height));
buffer.set_wrap(fs.raw(), to_wrap(wrapping));
buffer.set_rich_text(
    fs.raw(),
    spans.iter().enumerate().map(|(i, span)| {
        let attrs = to_attributes(span.font.unwrap_or(font))
            .metrics(...)
            .color(...)
            .letter_spacing(px / span_size);
        (span.text.as_ref(), attrs.metadata(i))
    }),
    &to_attributes(font),
    cosmic_text::Shaping::Advanced,
    None,
);
let min_bounds = align(&mut buffer, fs.raw(), align_x);
```

`Paragraph` хранит `Arc<cosmic_text::Buffer>` (не Buffer по значению), чтобы `fill_raw` мог получить `Weak<Buffer>`; resize идёт через `Arc::make_mut`.

Для рисования повторяем семантику `widget_text::draw`: `bounds.anchor(min_bounds, align_x, align_y)` → `renderer.fill_raw(Raw { buffer: Arc::downgrade(buffer), position: anchor, color, clip_bounds: *viewport })`.

Альтернативы:

- форк `iced_graphics` — запрещён `AGENTS.md`;
- оставить iced `Paragraph` без spacing — регрессия `letter-spacing` из конфига;
- свой `Renderer` с ассоциированным `Paragraph` — пришлось бы реализовывать весь renderer trait и всё равно рисовать через iced;
- вставлять пробелы/zero-width символы — ломает source-map, выделение и shaping.

### 2. Свой markdown-слой поверх pulldown-cmark 0.13

**Решение:** `src/markdown/`:

```
item.rs        Item, Bullet, Column, Row (+ cells())
text.rs        Text { source, spans }, parsed Span { source, style flags }, source_segments
parse.rs       pulldown events -> Vec<Item>, code_line_sources
settings.rs    Settings, Style, HeadingLevel, Alignment
viewer.rs      Viewer<'a, Message>
view.rs        view_with / item
list_view.rs   unordered_list / ordered_list
table_view.rs  table
quote_view.rs  quote
cull.rs        Cull
```

Разбор — один проход `Parser::new_ext(source, options).into_offset_iter()`, без incremental `Content` и без broken-link callback: файл парсится целиком в `Article`. `Text::source()` выводится из первого и последнего span; `source_segments()` строится по видимым span до стилизации, поэтому разбиение на script-runs и inline-код не меняет логические диапазоны.

Код-блоки: `code_line_sources` режет `Event::Text` по `split_inclusive('\n')`, срезает `\r`, source-диапазон указывает на содержимое без перевода строки; пустая строка тоже получает `Text`, чтобы не терять строку в selection-геометрии. Собственная подсветка кода приложением (`theme::Highlighter` на syntect/two-face) не меняется; вендоренный `iced_highlighter` не нужен.

`Text::spans(style)` кэширует `Arc<[text::Span]>` по `Style`, иначе `SpanStyler` перестанет попадать в кэш по `Arc::ptr_eq` и каждый кадр будет аллоцировать spans.

Альтернативы:

- оставить `iced_widget::markdown` — запрещено;
- handwritten GFM-парсер — отдельный, существенно больший change; риск разойтись с GFM выше, чем при использовании зрелого парсера;
- хранить source-диапазоны отдельной картой — дублирует данные и ломается на вложенных списках/таблицах.

### 3. Сборка блоков остаётся за проектом, iced-виджеты — примитивы

**Решение:** `DocumentView` реализует собственный `Viewer`; списки, цитата и таблица переносятся из вендоренного markdown в `list_view.rs`, `quote_view.rs`, `table_view.rs` почти один-к-одному, но с нашими `Item`/`Column`/`Row`. `view_with` строит `iced::widget::column(...)` с `settings.spacing`; `item` — `match` по `Item` и вызов `Viewer`.

Culling переносится в `Cull`:

- `update`: `CursorMoved` не пробрасывается, если `!layout.bounds().intersects(viewport)`;
- `mouse_interaction`: `Interaction::None`, если курсор не над `layout.bounds()`, дочерний виджет не опрашивается;
- всё остальное (нажатия, отпускания, draw, operate, overlay, layout) пробрасывается прозрачно.

`Cull` оборачивает результат `item(...)`, то есть каждый блок на всех уровнях: верхний column, пункты списков, ячейки таблиц, цитаты. Это семантика удалённого патча `row.rs`/`column.rs`, но в нашем коде.

Альтернативы:

- патчить row/column — запрещено;
- свои column/row — лишний риск геометрии и ресайза без выгоды;
- виртуализация списка блоков — отдельная задача, не требуется для удаления vendor.

### 4. Доступ к offset прокрутки — синхронная `Operation`, а не публичный `State`

**Решение:** новый `src/app/scroll_offset.rs` с `Access`-операцией. `WheelScroll::update` вызывает её прямо по своему content:

```rust
self.content.as_widget_mut().operate(
    &mut tree.children[0], layout, renderer, &mut Access::read()
);
```

- `scrollable(...)` получает `translation`; для вертикального направления с `Anchor::Start` `translation.y` — текущий clamped offset; для записи вызывается `state.scroll_to(AbsoluteOffset { x: None, y: Some(y) })`;
- `traverse` — no-op, поэтому вложенные scrollable (код-блоки, таблицы) не посещаются;
- `sync_scroll` читает offset после `content.update`, `apply_offset` пишет синхронно; сообщения приложения не публикуются.

Альтернативы:

- публичный `State` — это и есть удаляемый патч;
- `on_scroll(Viewport)` + `scroll_to` Task каждый кадр — лишний message-roundtrip, пересборка view и регрессия производительности, которую предыдущий change специально убрал;
- свой scrollable — большой риск двухпроходной раскладки при ресайзе.

Цена: `translation` у iced округлён (`round()`), поэтому внешняя синхронизация offset даёт погрешность ≤0.5 px. Принято: `apply_offset` пишет точное значение анимации, а чтение после drag/resize даёт позицию, совпадающую с нарисованной.

### 5. Ресайз: сохранить контракт iced scrollable и layout-кэшей

**Решение:** не менять механику `scrollable` и двухпроходную раскладку при появлении/исчезновении полосы. Новый `Paragraph` обязан реализовать те же переходы, что iced:

- `compare` возвращает `Shape` при смене шрифта/метрик/wrapping/shaping/alignment/version font system, `Bounds` при смене bounds, иначе `None`;
- `resize` вызывает `buffer.set_size(...)` и заново `align`, обновляя `min_bounds`;
- `TextLayout` сохраняет `LayoutChange::{Unchanged, Resized, Reshaped}`;
- `SelectableText::layout` и `InlineFlow::layout` регистрируют геометрию при `Resized`/`Reshaped`, а не только при `Reshaped`;
- `InlineFlow::sync` пересчитывает переносы при изменении `max_width` с допуском `WIDTH_EPSILON`;
- `SelectionSurface::layout` по-прежнему ставит `content_width` после layout детей.

После `Resized` `WheelScroll::update` пропускает событие в content, затем `sync_scroll` читает clamped offset и обновляет `Rc<Cell<f32>>`, который использует `SelectionSurface` для перевода окна в координаты контента.

### 6. Inline-движок в таблице, списке и цитате

Inline-движок остаётся контекстно-независимым. Таблица даёт ему только три вещи:

1. ширину лимита: iced table кладёт `Length::Shrink`-ячейку с максимумом `available.width - x`; `InlineFlow` берёт `limits.max().width` и перестраивает строки;
2. позицию: `set_origin(layout.bounds().position())`;
3. высоту строки: `min_bounds()` с vertical padding чипа; table берёт максимум по строке.

Сборка:

```
scrollable (vertical)
└ container
  └ markdown column
    └ Cull
      └ table
        └ scrollable (horizontal)
          └ iced::widget::table
            ├ header cell: column[ Cull[ paragraph -> SelectableText/InlineFlow ] ]
            └ body cell:   column[ Cull[ paragraph -> SelectableText/InlineFlow ] ]
```

`table_view` повторяет структуру вендоренного: `table::column(items(viewer, settings, &column.header), move |row: &Row| ...)`; ячейка — `items(viewer, settings, cells)`, где `items` — column из `Cull(item(...))`. Ячейки таблицы получают уникальный `block` = `text.source().start`; `SourceIndex` и `line_owners` уже обрабатывают несколько ячеек на одной исходной строке (тест `table_cells_on_one_row_share_source_line_and_full_width_rects`).

Ресайз таблицы: iced table пересчитывает `Limits` для ячеек; `InlineFlow::sync` видит новую ширину, перестраивает строки; `DocumentSelection` хранит `block + byte`, поэтому логический диапазон не меняется, а подсветка перерисовывается по новой геометрии.

### 7. Зависимости и фичи

**Решение:** удалить `[patch.crates-io]` и `vendor/`; убрать из `iced` фичи `markdown` и `highlighter`; `pulldown-cmark` остаётся прямым `0.13`; cosmic-text используется через re-export iced, как уже сделано в `selection/text_layout.rs`. Новых крейтов не добавляется.

## Risks / Trade-offs

- **`fill_raw` может отличаться от `fill_paragraph` по клипу/цвету/позиции** → визуальная проверка на фикстуре с letter-spacing, inline-кодом и ссылками; юнит-тест ширины spacing; при расхождении — корректировка `Paragraph::draw`, не модели.
- **Свой парсер разойдётся с вендоренным на metadata/task-list/broken-link** → перенос event-логики один-к-одному, тесты на все `Item`-варианты, source-диапазоны, soft/hard break, code-строки, пустые ячейки.
- **Округлённый `translation.y` (±0.5 px)** → принято; точное значение анимации пишется через `scroll_to`, чтение нужно только для внешней синхронизации.
- **Culling может скрыть hover/нажатие на границе viewport** → сравнение `layout.bounds().intersects(viewport)`; нажатия не фильтруются, только `CursorMoved`/`mouse_interaction`; визуальная проверка и perf-прогон.
- **Ресайз** → сохраняем iced scrollable, `Bounds`-переход paragraph, регистрацию геометрии на `Resized`, порядок `content_width`; отдельный sandbox-сценарий «ресайз при активном выделении».
- **Объём своего кода** (~1000–1300 строк) → цена отсутствия форка; модули разбиты по концептам, тесты переносятся вместе с кодом.
- **Производительность `fill_raw`** → buffer переиспользуется в state; кэш `Text::spans` по `Style`; `sandbox-perf.sh` должен остаться <100 % CPU на wheel/drag.
- **Ширина ячейки таблицы через `Shrink` + остаток ширины** → поведение iced table сохраняется; «естественная ширина без переноса» не входит в change.

## Migration Plan

1. Ввести `src/text/` и перевести `selection`/`theme` на собственные `Span`/`Paragraph`; временно iced markdown ещё присутствует.
2. Ввести `src/markdown/` (модель, парсер, view) и переключить `Article`/`DocumentView`/`Viewer`/`ViewerText`/`Style` на него.
3. Заменить culling на `Cull`; заменить доступ к offset на `Access`-операцию.
4. Удалить `[patch.crates-io]`, `vendor/`, фичи `markdown`/`highlighter`; перевести `rich_text` на `Label`; убрать оставшиеся импорты iced markdown.
5. Прогнать `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`.
6. Прогнать sandbox: GFM-фикстура, inline-чипы, таблица, code-блок, изображение; ресайз с активным выделением; `sandbox-perf.sh`.

Rollback: change изолирован в одном коммите/ветке; возврат коммита восстанавливает `vendor/` и патчи. Отдельных миграций данных нет.

## Open Questions

Нет. Вопрос «делать ли своими `column`/`row`/`scrollable`/`table`» решён: остаются iced. Вопрос «свой ли handwritten-парсер» решён: `pulldown-cmark` остаётся front-end.
