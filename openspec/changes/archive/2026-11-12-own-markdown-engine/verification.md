# Verification: own-markdown-engine

## Commands

- `direnv exec . cargo fmt --check` — green.
- `direnv exec . chrt -i 0 nice -n 19 ionice -c 3 cargo test -j 10` — 213 passed, 0 failed (baseline 183).
- `direnv exec . chrt -i 0 nice -n 19 ionice -c 3 cargo clippy --all-targets -j 10 -- -D warnings` — green.
- `grep -rn 'iced::widget::markdown\|iced::widget::text::Span\|rich_text\|scrollable::State' src` — empty.
- `grep -rn 'vendor/\|patch.crates-io\|iced_highlighter' Cargo.toml Cargo.lock src scripts README.md` — no source or manifest references; `Cargo.lock` keeps only the normal crates.io `iced_widget` dependency name.
- `openspec validate own-markdown-engine --type change` — valid.

## Sandbox

- `scripts/sandbox.sh target/acceptance.md` — hardware adapter `AMD Radeon Graphics (RADV RENOIR)`; screenshot shows headings, wrapped paragraph, nested list, ordered list, task list, quote with inline chip, table with inline chip, Rust code block, PNG image and rule. Inline chips keep background, padding and border on one line; no background spills onto neighbours.
- Resize with active selection: drag from the gutter across blocks and an inline chip, `Ctrl+C`, resize the headless output from 1280x800 to 900x600, `Ctrl+C` again. Highlight is repainted at new line positions; clipboard is identical before and after (`754` bytes).
- Scrolling: `scripts/sandbox-perf.sh target/acceptance.md` — `wheel_frame_changed=1`, `drag_frame_changed=1`, `wheel_cpu_percent=1.5`, `drag_cpu_percent=4.5`.
- Reload: append and atomic prepend (`sed -i`) to the open document preserve the scroll offset; the appended case gives identical before/after frames. Truncate-and-rewrite through `cat > file` can expose a transient empty document to the debounced watcher; that is the watcher/reload path, not the scroll access.

## Review

### 1. Files

- `src/text/mod.rs` — index only: private `mod label; mod paragraph; mod span;`, sorted `pub use`.
- `src/text/span.rs` — `Highlight`, `Span`; one topic: styled span value.
- `src/text/paragraph.rs` — `TextConfig`, `Difference`, `Paragraph`, `Internal`, font-system guards; one topic: cosmic-text paragraph layout, measurement and drawing.
- `src/text/label.rs` — `Label`, `LabelState`, `layout_label`; one topic: non-interactive label widget.
- `src/markdown/item.rs` — `Item`, `Bullet`, `Column`, `Row`, `HeadingLevel`, `Alignment`; one topic: parsed markdown model.
- `src/markdown/text.rs` — `Text`, `SourceSegment`, parsed `Span`; one topic: parsed text with source mapping and style cache.
- `src/markdown/parse.rs` — `parse`, `options`, `Parser`, `Scope`, `List`, `code_line_sources`; one topic: pulldown-cmark event fold.
- `src/markdown/settings.rs` — `Settings`, `Style`; one topic: rendering parameters.
- `src/markdown/cull.rs` — `Cull`, `skips`, `forwards_interaction`; one topic: viewport culling wrapper.
- `src/markdown/view.rs` — `view_with`, `item`, `items`; one topic: item-to-element dispatch.
- `src/markdown/viewer.rs` — `Viewer`, default `heading`, `paragraph`, `code_block`; one topic: view strategy.
- `src/markdown/list_view.rs` — `unordered_list`, `ordered_list`; one topic: list layout.
- `src/markdown/quote_view.rs` — `quote`; one topic: quote layout.
- `src/markdown/table_view.rs` — `table`, `horizontal`; one topic: table layout.
- `src/app/scroll_offset.rs` — `Mode`, `Access`; one topic: synchronous scroll offset operation.
- Changed: `article`, `uri`, `selection::*`, `theme::*`, `app::view`, `app::wheel_scroll`, `app::message`, `lib`.
- No logic in `mod.rs`/`main.rs`; tests live at the bottom of their files.

### 2. Entities

| Entity | Invariant / behaviour | Kind |
|---|---|---|
| `TextConfig` | defaults for alignment; copied, compared | value type, public fields |
| `Difference` | three comparison outcomes | value enum |
| `Paragraph` | buffer, metrics, alignment, bounds; constructor builds buffer | handle with private `Arc<Internal>` |
| `Label` | non-interactive rendering; builder setters | widget |
| `Item` | parsed markdown variants | value enum |
| `Column` | header and alignment | value type, public fields |
| `Row` | private cells, `cells()` reference | container |
| `Bullet` | point/task payload, task transition | value enum with methods |
| `SourceSegment` | logical and source range | value type, `PartialEq` |
| `Text` | private spans, source range, per-style span cache | container + cache |
| parsed `Span` | text, source, style flags | subsystem value, `pub(super)` fields |
| `Settings`, `Style` | rendering parameters | value types, public fields |
| `Cull` | transparent child wrapper | widget |
| `Access` | read/write scroll offset, no traversal | operation |
| `Parser` | event-fold state producing `Vec<Item>` | parser state with methods |
| `LabelState` | paragraph and span cache for `Label` | widget state |

No new type name repeats an existing subsystem type; parsed `markdown::text::Span` is private and distinct from `text::Span`.

### 3. Duplicates and state

- `Diff`/`LayoutChange`: single mapping in `selection::text_layout`; `Paragraph::compare` is the only comparison source.
- `heading_size`: single match in `Settings::heading_size`; `theme::style::heading_size` was removed.
- `items` is a named slice entry for table cells and delegates to `view_with`; one assembly body.
- No auxiliary boolean maps or index cycles were introduced; parser scopes are the data model.
- Span cache in `Text` is keyed by `Style` and returns the same `Arc`; `SpanStyler`/`InlineFlow` caches keep `Arc::ptr_eq` valid.

### 4. Flow

- Parse: markdown `&str` → `Vec<Item>`; every block carries `Text` with source ranges, code lines carry per-line ranges.
- View: `&[Item]` + `Settings` → `Element`; `item` dispatches through `Viewer` and wraps each block in `Cull`.
- Selection: spans → `Paragraph`/`InlineFlow` → `VisualLine` → `DocumentGeometry` → `DocumentSelection`; no data flows back into the parser or view.
- Scroll: `Access` reads the scrollable translation and writes `AbsoluteOffset`; the animation state stays inside `WheelScroll`.

### 5. Function bodies

- Guards return early (`let ... else`, `return`); no deep nested `if`.
- Branches over behaviour are `match` (`Item`, `Mode`, `Difference`, `Scope`, tag/event dispatch).
- Arithmetic and comparisons sit in named helpers (`vertical_limit`, `captures_wheel`, `horizontal`, `code_line_sources`).
- `Paragraph::new`, `Parser::apply/start/end` and the view builders are flat step sequences; no `unwrap`/`expect` in production code.

### 6. Tests

- `text::paragraph`: letter spacing widens measured text; `Bounds`/`Shape`/`None` transitions; stable draw anchor.
- `text::label`: shrink layout returns minimum bounds; repeated layout is `None`.
- `text::span`: letter spacing, background, border and padding survive the builders.
- `markdown::parse`: all `Item` variants, nested list, quote, table alignment and cells, task markers, metadata skip.
- `markdown::article`: empty source, plain/soft/hard break ranges, CRLF, multi-backtick code, several text events, empty code line, block starts unique and non-overlapping.
- `markdown::text`: style cache `Arc` identity, empty text, inline-code highlight.
- `markdown::view`: all item variants, nested list/quote, mixed-alignment table with an empty cell build without panic.
- `markdown::cull`: `CursorMoved` skipped outside and forwarded inside the viewport; other events pass; interaction is `None` outside bounds.
- `app::scroll_offset`: read and write against a dummy `operation::Scrollable`.
- Preserved suites: text layout, inline atomize/layout/state, selection, geometry, theme styler/highlighter/viewer, scroll and wheel scroll.
