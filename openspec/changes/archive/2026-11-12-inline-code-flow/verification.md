# Verification

Change: `inline-code-flow`
Date: 2026-11-12
Environment: Linux, X11 (Xvfb), debug build, embedded fonts.

## Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo test` | 134 passed, 0 failed |

## Visual checks

Fixture: heading, paragraph with source spaces, punctuation, two chips, long inline code, list, quote, table, code block.

- Default `padding 4`, `radius 4`: chip background surrounds the code word with 4 px padding; source spaces remain outside the border, and punctuation touches the border.
- `а   `код`   б`: all three source spaces are preserved on both sides of the chip.
- `letter-spacing 3`, `radius 12` via `MDRS_CONFIG`: glyph spacing inside inline code increases, neighboring words and body text keep their own spacing; corners are visibly rounder.
- Narrow window: chips move to the next line intact; a long word in a block with inline code continues across lines; a chip wider than the text area is clipped at the text bounds instead of painting over padding.
- Selection: dragging from a word through a chip to the next word highlights the whole range, including spaces. `Ctrl+C` returned `баться рылом в ламинат` (drag started inside the first glyph, so the leading `е` was excluded) — source spaces around the chip are preserved.
- Resize with active selection: the same logical text was copied after resize, and the highlight was redrawn at the new line positions.
- Code block selection and copy still work; blocks without inline code keep the old `SelectableText` path.

Known limitation: a chip wider than the available text width is clipped because an atomic chip cannot be split; bidi/CJK line breaking inside blocks containing inline code remains simpler than cosmic-text's full algorithm.

## Review (AGENTS.md)

1. **Files.** New files: `inline_atom.rs` (atom/box data), `inline_atomize.rs` (spans → atoms), `inline_layout.rs` (atoms → lines, hit-testing, selection geometry), `inline_state.rs` (interaction state), `inline_flow.rs` (widget), `inline_render.rs` (draw phases), `font_metrics.rs` (font metrics + cache). Each has one theme: atom model, atomization, flow layout, interaction, widget contract, rendering, metrics. No logic added to `mod.rs`; tests sit at the bottom of their files. Operations live with the type whose fields they read; no free helper reads only foreign state.
2. **Entities.** `InlineConfig` is a value type with public fields and no invariant. `Chip`/`TextBody` have constructors and private fields; `Atom` is the cross-file atom contract. `LineItem`, `LeadingSpace`, `Fragment`, `FontMetrics` are named value types instead of tuples. `SpanCache` is a type alias for a homogeneous map. No duplicate type names within a subsystem.
3. **Duplication/state.** Chip and text phases share atom/line traversal, not duplicated algorithms. `pending_space_range` reflects the actual pending whitespace atom; no boolean flags or auxiliary maps track layout state.
4. **Flow.** Input spans → atoms → lines → draw/selection. Phases take values and return values; no side channel writes into foreign state. Atom measurement is cached and resize only re-runs line breaking.
5. **Bodies.** Guards are early returns; loops are flat; comparisons and arithmetic live in named helpers (`metrics`, `fragments`, `finish_line`, `line_at`). No `unwrap`/`expect` in production code; panics remain test-only.
6. **Tests.** 134 tests pass, including atomization boundaries, chip atomic wrapping, preserved spaces, resize reflow, selection/copy across chips, letter-spacing width, font metrics, and configuration parsing. No test was changed to force a green run except the existing default-padding assertion, which was updated to the new specified default of 4.
