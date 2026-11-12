# Tasks: own-markdown-engine

## 1. Text layer

- [x] 1.1 Создать `src/text/mod.rs` с приватными `mod label; mod paragraph; mod span;` и `pub use` своих типов; `src/lib.rs` подключает `mod text;`. Проверка: `direnv exec . chrt -i 0 nice -n 19 ionice -c 3 cargo check` проходит, в `src/text/` нет логики в `mod.rs`.
- [x] 1.2 Реализовать `text::Span` с полями `text`, `size`, `line_height`, `letter_spacing`, `font`, `color`, `link`, `highlight`, `padding`, `underline`, `strikethrough` и конструкторами/сеттерами, используемыми проектом. Проверка: юнит-тест собирает span с `letter_spacing`, `background`, `border`, `padding` и читает поля.
- [x] 1.3 Реализовать `text::Paragraph` на `cosmic_text::Buffer` через публичные `font_system`, `to_attributes`, `to_wrap`, `align`: `new`, `resize`, `compare`, `min_bounds`, `span_bounds`, `hit_span`, `buffer`, `align_x`, `align_y`; `letter_spacing` применять как `px / span_size` в `Attrs`. Проверка: тест «letter spacing increases measured width» на `"ab"` с 10 px и spacing 2 px даёт ширину больше на >1.5 px.
- [x] 1.4 Реализовать `Paragraph::draw` через `graphics::text::Renderer::fill_raw`, `Raw { buffer: Arc::downgrade(...), position: bounds.anchor(...), color, clip_bounds: *viewport }`; буфер хранить как `Arc<Buffer>`, arena держать в state виджета. Проверка: `cargo check` и юнит-тест, что `min_bounds`/anchor не меняются при повторном draw-вызове.
- [x] 1.5 Реализовать `text::Label` — неинтерактивный виджет: `layout` через `Paragraph::compare/resize`, `draw` через `Paragraph::draw`, без `mouse_interaction`. Проверка: юнит-тест layout возвращает `min_bounds` при `Length::Shrink`; повторный layout с теми же bounds даёт `Difference::None`.
- [x] 1.6 Убедиться, что `Paragraph::compare` различает `Shape`/`Bounds`/`None` и `resize` перекладывает buffer. Проверка: тест на три перехода: смена bounds → `Bounds`, смена size/font → `Shape`, повтор → `None`.

## 2. Markdown model and parser

- [x] 2.1 Создать `src/markdown/item.rs`: `Item`, `Bullet` (point/task), `Column`, `Row` с `cells()`, `HeadingLevel`, `Alignment`; `Row` хранит `Vec<Vec<Item>>` приватно. Проверка: тест fixture содержит все варианты `Item`, `Row::cells()` доступен, task-вариант сохраняет `done`.
- [x] 2.2 Создать `src/markdown/text.rs`: `Text { source: Range<usize>, spans }`, приватный parsed `Span` с source и флагами strong/emphasis/strikethrough/code/link, `source()`, `source_segments()`, кэш `spans(style) -> Arc<[text::Span]>`. Проверка: тестами article: plain span, soft break, inline code source, пустой вход; повторный `spans(style)` возвращает тот же `Arc`.
- [x] 2.3 Реализовать `parse.rs` на `pulldown-cmark 0.13` с `into_offset_iter`: headings, paragraphs, lists (вложенные), task lists, quotes, code blocks, images, tables, metadata skip, soft/hard break, inline code, links, emphasis/strikethrough. Проверка: fixture-тест всех `Item`-вариантов, тест вложенного списка/цитаты/таблицы.
- [x] 2.4 Реализовать `code_line_sources`: `split_inclusive('\n')`, срез `\r`, source-диапазон без перевода строки, пустая строка получает `Text`. Проверка: тест `code_lines_exclude_fence_and_header` даёт `lines[0].source() == 8..20` и `&source[range] == "fn main() {}"`; тест пустой строки кода.
- [x] 2.5 Перевести `Article::parse_with_source` на собственный `parse`; `Article` хранит `Vec<Item>` и два `SourceIndex`. Проверка: тесты `markdown/article.rs`: пустой источник, copy index при resolved source, `source_index_matches_parsed_source`.
- [x] 2.6 Проверить границы source-map: CRLF, soft break, inline code с backticks, несколько текстовых событий в одном блоке, ячейки таблицы на одной строке. Проверка: тесты `article` и `block_source` зелёные, source-диапазоны не пересекаются у разных блоков.

## 3. Markdown view and culling

- [x] 3.1 Создать `src/markdown/settings.rs` (`Settings`, `Style`, re-export `HeadingLevel`/`Uri`), `viewer.rs` (собственный `Viewer<'a, Message>`) и `view.rs` (`view_with`, `item`, `items`); `item` оборачивает результат в `Cull`. Проверка: `cargo check`; тест, что `view_with` принимает все `Item`-варианты без паники на тестовом `Message`.
- [x] 3.2 Реализовать `list_view.rs` (unordered/ordered, маркер `text`, `checkbox` для task, вложенный `view_with`) и `quote_view.rs` (`row` с `rule::vertical`, вложенный column). Проверка: тест fixture со вложенным списком/цитатой собирается; `cargo test` для markdown-тестов.
- [x] 3.3 Реализовать `table_view.rs`: `iced::widget::table`, `table::column` по заголовку, closure по `row.cells().get(i)`, align_x из `Alignment`, горизонтальный `scrollable`. Проверка: тест таблицы с тремя колонками и разным alignment; ячейки без значения дают пустой элемент.
- [x] 3.4 Реализовать `Cull`: пропуск `CursorMoved` при `!layout.bounds().intersects(viewport)`, `Interaction::None` при курсоре вне bounds, прозрачный проброс остальных событий/draw/operate/overlay/layout. Проверка: юнит-тест счётчика вызовов дочернего виджета на `CursorMoved` внутри/вне viewport и `mouse_interaction` вне bounds.
- [x] 3.5 Перевести `theme/document_view.rs` на собственный `Viewer` и типы; `DocumentView::paragraph/heading/code_block` продолжают строить `BlockSource` из `Text::source_segments`. Проверка: тест, что block id табличных ячеек уникальны и совпадают с `text.source().start`.
- [x] 3.6 Проверить порядок блоков и вложенность: списки, цитата, таблица, изображение, rule; block id идут в порядке исходника. Проверка: тест `Article` + `BlockSource` на fixture, все block id возрастают в порядке обхода `Item`.

## 4. Selection and theme adaptation

- [x] 4.1 Перевести `selection/text_layout.rs` на `text::Paragraph` и `text::TextConfig`/собственный config; сохранить `LayoutChange` и `visual_lines`. Проверка: тесты `text_layout`: `Unchanged`/`Resized`/`Reshaped`, `point_in_text`, `logical_at`.
- [x] 4.2 Перевести `SelectableText::draw` на `Paragraph::draw` вместо `widget_text::draw`; `TextState::paragraph` возвращает свой `Paragraph`. Проверка: `cargo test` selection-тестов; визуально текст, подсветка inline-кода и underline/strikethrough не смещаются.
- [x] 4.3 Перевести `InlineAtom`, `InlineAtomize`, `InlineLayout`, `InlineState`, `InlineRender` на собственные `Span`/`Paragraph`; измерение чипов и `metrics` сохранить. Проверка: тесты `inline_atomize`, `inline_layout`, `inline_state`, включая `letter_spacing_increases_measured_width` и `resize_reflows_lines_without_splitting_chip`.
- [x] 4.4 Перевести `theme/styler.rs`, `theme/viewer_text.rs`, `theme/highlighter.rs` на `crate::text::Span`; `letter_spacing` роли по-прежнему пишется в span. Проверка: тесты styler («role letter spacing is written to spans»), highlighter (цвета и fallback), viewer_text.
- [x] 4.5 Заменить `rich_text` в `app/view.rs` (экран сообщения) и `theme/viewer.rs` (alt картинки) на `text::Label`; `Span::new(...).letter_spacing(...)` использовать на своём типе. Проверка: `cargo check`; визуально сообщение и alt-текст рисуются с настроенным spacing.
- [x] 4.6 Удалить все импорты `iced::widget::markdown` и `iced::widget::text::Span`; `Uri` брать из `crate::markdown`. Проверка: `grep -rn 'iced::widget::markdown\|iced::widget::text::Span' src` пуст; `cargo check` проходит.

## 5. Scroll offset access

- [x] 5.1 Создать `src/app/scroll_offset.rs`: `Access`-операция с `Mode::{Read, Write(f32)}`, `traverse` — no-op, `scrollable` читает `translation.y` или вызывает `state.scroll_to(AbsoluteOffset { x: None, y: Some(y) })`. Проверка: юнит-тест с dummy `operation::Scrollable` проверяет read и write без iced scrollable.
- [x] 5.2 Перевести `WheelScroll::sync_scroll` на `Access::read`, `apply_offset` — на `Access::write`; `State` больше не downcast-ится. Проверка: `grep -rn 'scrollable::State' src` пуст; тесты `wheel_scroll` и `scroll` зелёные.
- [x] 5.3 Проверить, что анимация колеса не публикует сообщения и не пересобирает view: в `WheelScroll::update` на wheel/redraw только `shell.request_redraw()` и `Access::write`. Проверка: review diff; `sandbox-perf.sh` wheel CPU <100 %.

## 6. Dependency cleanup

- [x] 6.1 Удалить из `Cargo.toml` фичи iced `markdown` и `highlighter`; `pulldown-cmark` оставить прямым `0.13`; новых крейтов не добавлять. Проверка: `cargo check`; `grep -n 'markdown\|highlighter' Cargo.toml` не содержит фич iced.
- [x] 6.2 Удалить `[patch.crates-io]` и каталог `vendor/`; обновить `Cargo.lock`. Проверка: `direnv exec . chrt -i 0 nice -n 19 ionice -c 3 cargo check` и `cargo test` собираются из crates.io; `git status` не содержит `vendor/`.
- [x] 6.3 Проверить, что в проекте не осталось ссылок на вендоренные крейты и патчи. Проверка: `grep -rn 'vendor/\|patch.crates-io\|iced_widget\|iced_highlighter' Cargo.toml Cargo.lock src scripts README.md` — только допустимые упоминания в `AGENTS.md`/истории, кода нет.
- [x] 6.4 Обновить документацию, если она утверждает, что markdown-рендер идёт через iced: `README.md`, комментарии конфига, `config.example.kdl`. Проверка: `grep -rni 'iced.*markdown\|markdown.*iced' README.md config.example.kdl` не даёт ложных утверждений.

## 7. Verification and acceptance

- [x] 7.1 `direnv exec . chrt -i 0 nice -n 19 ionice -c 3 cargo fmt --check`; `cargo test`; `cargo clippy --all-targets -- -D warnings`. Проверка: все три команды зелёные, число тестов не меньше прежнего (183) плюс новые тесты text/parser/cull/scroll_offset.
- [x] 7.2 Юнит-тесты покрывают границы: пустой markdown, CRLF, soft/hard break, вложенные списки, пустые ячейки таблицы, пустые строки code-блока, letter-spacing, resize reflow, cull внутри/вне viewport, read/write offset. Проверка: `cargo test` с перечисленными именами тестов.
- [x] 7.3 Визуальная приёмка `scripts/sandbox.sh` на фикстуре: заголовки, абзацы, вложенный список, цитата, таблица с inline-кодом, многострочный code-блок, изображение, длинный переносимый абзац. Проверка: `target/screenshot.png`; inline-чипы целы, фон и текст не перекрывают соседние строки.
- [x] 7.4 Ресайз с активным выделением: запустить sandbox, протянуть выделение через несколько блоков и чип, изменить размер окна (`xdotool windowsize`), проверить подсветку на новых переносах и Ctrl+C. Проверка: после ресайза копируется тот же исходный markdown, подсветка на новых позициях.
- [x] 7.5 Прокрутка: колесо, серия щелчков, границы документа, scrollbar drag, live reload. Проверка: `scripts/sandbox-perf.sh` печатает `wheel_frame_changed=1`, `drag_frame_changed=1`, wheel/drag CPU <100 %; позиция после reload восстанавливается.
- [x] 7.6 Ревью по AGENTS.md: файлы/сущности/дубли/поток/тело/тесты; записать результат в `verification.md` при apply. Проверка: каждый шаг ревью имеет перечисление или ответы, `openspec validate --change own-markdown-engine` проходит.
