## Why

Markdown-рендер, карта исходного текста, межбуквенный интервал и доступ к позиции прокрутки сейчас держатся на трёх вендоренных крейтах iced и пяти патчах в них. `AGENTS.md` уже запрещает `vendor/` и `[patch.crates-io]`, поэтому обновление iced, замена markdown-рендера или исправление одного патча без форка невозможны.

## What Changes

- Удалить `vendor/iced_core`, `vendor/iced_graphics`, `vendor/iced_widget` и секцию `[patch.crates-io]` из `Cargo.toml`.
- Ввести собственный текстовый слой: `Span` с `letter_spacing`, `Paragraph` на cosmic-text через публичные `font_system`/`align`/`fill_raw`, неинтерактивный `Label` для экрана сообщения и alt-текста.
- Ввести собственный markdown-слой: разбор `pulldown-cmark 0.13` в `Item`/`Text`/`Span` с source-диапазонами, `source_segments`, диапазонами строк кода, `Row::cells`, собственные `Viewer`, `view_with`, списки, цитата и таблица.
- Заменить патч `row.rs`/`column.rs` на прозрачный `Cull`-виджет: `CursorMoved` и `mouse_interaction` не доходят до блоков вне viewport и вне курсора.
- Заменить публичный `scrollable::State` на синхронную `Operation`-операцию чтения и записи offset через `Widget::operate`; анимация колеса остаётся в виджете и не публикует сообщения.
- Убрать фичи iced `markdown` и `highlighter`; `rich_text` и `iced::widget::text::Span` в коде приложения больше не используются.
- Сохранить пользовательское поведение без изменений: GFM-рендер, inline-чипы, `letter-spacing`, выделение и копирование исходного markdown, перестройку при ресайзе, колесо, drag и полосы прокрутки.
- **BREAKING** для сборки: `vendor/` и path-патчи удаляются; крейты iced берутся только из crates.io.

## Capabilities

### New Capabilities

Нет. Поведение системы не меняется; это рефакторинг зависимостей и внутреннего движка.

### Modified Capabilities

Нет. `markdown-viewer`, `text-selection` и `configuration` сохраняют текущие требования; существующие сценарии ресайза, inline-кода, выделения и прокрутки остаются критериями приёмки.

Изменение объявлено в `.openspec.yaml` как `skip_specs: true`: спека не заводится ради зелёной валидации.

## Impact

- `Cargo.toml`, `Cargo.lock`, удаление `vendor/`.
- Новый `src/text/`: `span.rs`, `paragraph.rs`, `label.rs`.
- `src/markdown/`: собственные `item.rs`, `text.rs`, `parse.rs`, `settings.rs`, `viewer.rs`, `view.rs`, `list_view.rs`, `table_view.rs`, `quote_view.rs`, `cull.rs`; существующие `article.rs`, `source_index.rs`, `resolve.rs`, `uri.rs` адаптируются.
- `src/selection/`: `TextLayout`, `InlineAtomize`, `InlineRender`, `SelectableText`, `InlineFlow`, `TextState`, `InlineState` переходят на собственные `Span`/`Paragraph`.
- `src/theme/`: `document_view`, `viewer`, `viewer_text`, `style`, `styler`, `highlighter`, `code_block` переводятся на собственные типы; `rich_text` заменяется на `Label`.
- `src/app/`: `view`, `message`, `wheel_scroll`, новый `scroll_offset`.
- Проверка: `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `scripts/sandbox.sh`, `scripts/sandbox-perf.sh`; отдельный сценарий ресайза с активным выделением.
