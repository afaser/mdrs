use std::ops::Range;
use std::sync::Arc;

use iced::advanced::graphics::text::Paragraph as TextParagraph;
use iced::advanced::text::{Alignment, Paragraph as _, Text, Wrapping};
use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Background, Border, Color, Padding, Size};

const MEASURE_BOUNDS: f32 = 1_000_000.0;

use crate::selection::inline_atom::{Atom, AtomBody, Chip, InlineConfig, TextBody, TextStyle};

#[derive(Debug, Clone)]
struct Piece {
    span: Span<'static, Uri>,
    local: Range<usize>,
    global: Range<usize>,
}

#[derive(Debug, Clone)]
enum Raw {
    Piece(Piece),
    Chip {
        span: Span<'static, Uri>,
        range: Range<usize>,
    },
    Space {
        range: Range<usize>,
        count: usize,
    },
}

pub(super) fn logical_text(spans: &[Span<'static, Uri>]) -> String {
    let mut text = String::new();

    for span in spans {
        text.push_str(span.text.as_ref());
    }

    text
}

pub(super) fn atomize(spans: &Arc<[Span<'static, Uri>]>, config: InlineConfig) -> Vec<Atom> {
    let raw = split(spans);
    let mut atoms = Vec::new();
    let mut pending: Vec<Piece> = Vec::new();

    for item in raw {
        match item {
            Raw::Piece(piece) => pending.push(piece),
            Raw::Space { range, count } => {
                flush_text(&mut pending, &mut atoms, config);
                atoms.push(Atom {
                    range,
                    body: AtomBody::Space {
                        width: config.space * count as f32,
                    },
                });
            }
            Raw::Chip { span, range } => {
                flush_text(&mut pending, &mut atoms, config);
                atoms.push(chip_atom(span, range, config));
            }
        }
    }

    flush_text(&mut pending, &mut atoms, config);
    atoms
}

fn split(spans: &[Span<'static, Uri>]) -> Vec<Raw> {
    let mut raw = Vec::new();
    let mut start = 0;

    for span in spans {
        let text = span.text.as_ref();
        let end = start + text.len();

        if span.highlight.is_some() {
            raw.push(Raw::Chip {
                span: span.clone(),
                range: start..end,
            });
            start = end;
            continue;
        }

        split_span(span, start, &mut raw);
        start = end;
    }

    raw
}

fn split_span(span: &Span<'static, Uri>, offset: usize, raw: &mut Vec<Raw>) {
    let text = span.text.as_ref();
    let mut run_start = 0;
    let mut run_space = text.chars().next().is_some_and(char::is_whitespace);

    for (index, character) in text.char_indices() {
        let space = character.is_whitespace();

        if space == run_space {
            continue;
        }

        push_segment(span, offset, run_start..index, run_space, raw);
        run_start = index;
        run_space = space;
    }

    push_segment(span, offset, run_start..text.len(), run_space, raw);
}

fn push_segment(
    span: &Span<'static, Uri>,
    offset: usize,
    local: Range<usize>,
    space: bool,
    raw: &mut Vec<Raw>,
) {
    if local.is_empty() {
        return;
    }

    let global = offset + local.start..offset + local.end;

    if space {
        let count = span
            .text
            .get(local.clone())
            .map_or(1, |text| text.chars().count());
        raw.push(Raw::Space {
            range: global,
            count,
        });
        return;
    }

    raw.push(Raw::Piece(Piece {
        span: span.clone(),
        local,
        global,
    }));
}

fn flush_text(pending: &mut Vec<Piece>, atoms: &mut Vec<Atom>, config: InlineConfig) {
    if pending.is_empty() {
        return;
    }

    let first = pending[0].global.start;
    let last = pending[pending.len() - 1].global.end;
    let mut spans = Vec::with_capacity(pending.len());
    let mut styles = Vec::new();
    let mut local_start = 0;

    for piece in pending.drain(..) {
        let fragment = piece.span.text.get(piece.local.clone()).map(str::to_owned);
        let Some(fragment) = fragment else {
            continue;
        };
        let mut span = piece.span;
        let style_end = local_start + fragment.len();
        span.text = std::borrow::Cow::Owned(fragment);
        styles.push(TextStyle {
            range: local_start..style_end,
            underline: span.underline,
            strikethrough: span.strikethrough,
            link: span.link.clone(),
            color: span.color,
            size: span.size.unwrap_or(config.size).0,
        });
        local_start = style_end;
        spans.push(span);
    }

    let paragraph = measure(&spans, config);
    let (width, ascent, descent) = metrics(&paragraph, config);

    atoms.push(Atom {
        range: first..last,
        body: AtomBody::Text(TextBody::new(paragraph, width, ascent, descent, styles)),
    });
}

fn chip_atom(span: Span<'static, Uri>, range: Range<usize>, config: InlineConfig) -> Atom {
    let background = span
        .highlight
        .map(|highlight| highlight.background)
        .unwrap_or(Background::Color(Color::TRANSPARENT));
    let border = span
        .highlight
        .map_or(Border::default(), |highlight| highlight.border);
    let padding = span.padding;
    let font = span.font.unwrap_or(config.font);
    let size = span.size.unwrap_or(config.size);
    let mut content = span;
    content.highlight = None;
    content.padding = Padding::default();
    content.underline = false;
    content.strikethrough = false;
    let paragraph = measure(&[content], config);
    let (content_width, paragraph_ascent, paragraph_descent) = metrics(&paragraph, config);
    let metrics = crate::theme::FontMetrics::of(font, size);
    let content_ascent = metrics.map_or(paragraph_ascent, |metrics| {
        paragraph_ascent.max(metrics.ascent())
    });
    let content_descent = metrics.map_or(paragraph_descent, |metrics| {
        paragraph_descent.max(metrics.descent())
    });

    Atom {
        range,
        body: AtomBody::Chip(Box::new(Chip::new(
            paragraph,
            content_width,
            content_ascent,
            content_descent,
            padding,
            background,
            border,
        ))),
    }
}

fn measure(spans: &[Span<'static, Uri>], config: InlineConfig) -> TextParagraph {
    let text = Text {
        content: spans,
        bounds: Size::new(MEASURE_BOUNDS, MEASURE_BOUNDS),
        size: config.size,
        line_height: config.line_height,
        font: config.font,
        align_x: Alignment::Left,
        align_y: iced::alignment::Vertical::Top,
        shaping: config.shaping,
        wrapping: Wrapping::None,
    };

    TextParagraph::with_spans(text)
}

fn metrics(paragraph: &TextParagraph, config: InlineConfig) -> (f32, f32, f32) {
    let width = paragraph.min_bounds().width;
    let Some(run) = paragraph.buffer().layout_runs().next() else {
        return (width, config.line_height.to_absolute(config.size).0, 0.0);
    };

    (
        width,
        run.line_y,
        run.line_top + run.line_height - run.line_y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::text::{LineHeight, Shaping};
    use iced::widget::markdown::Uri;
    use iced::{Font, Pixels};

    fn config() -> InlineConfig {
        crate::theme::isolate_system_fonts();

        InlineConfig {
            size: Pixels(17.0),
            font: Font::default(),
            line_height: LineHeight::Relative(1.3),
            shaping: Shaping::Advanced,
            space: 4.0,
        }
    }

    fn code(text: &str) -> Span<'static, Uri> {
        Span::new(text.to_owned()).background(Color::from_rgb8(0x31, 0x32, 0x44))
    }

    fn atomize_text(spans: Vec<Span<'static, Uri>>) -> (Vec<Atom>, String) {
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(spans.into_boxed_slice());
        (atomize(&spans, config()), logical_text(&spans))
    }

    #[test]
    fn plain_text_splits_into_words_and_one_space() {
        let (atoms, logical) = atomize_text(vec![Span::new("alpha   beta".to_owned())]);

        assert_eq!(logical, "alpha   beta");
        assert_eq!(atoms.len(), 3);
        assert!(!atoms[0].is_space());
        assert!(atoms[1].is_space());
        assert!(!atoms[2].is_space());
    }

    #[test]
    fn spaces_next_to_chip_are_preserved() {
        let (atoms, logical) = atomize_text(vec![
            Span::new("ебаться ".to_owned()),
            code("рылом"),
            Span::new(" в ламинат".to_owned()),
        ]);

        assert_eq!(logical, "ебаться рылом в ламинат");
        assert_eq!(atoms.len(), 7);
        assert!(atoms[1].is_space());
        assert!(atoms[2].is_chip());
        assert!(atoms[3].is_space());
        assert_eq!(atoms.iter().filter(|atom| atom.is_space()).count(), 3);
    }

    #[test]
    fn chip_next_to_punctuation_has_no_gap() {
        let (atoms, logical) = atomize_text(vec![
            Span::new("(".to_owned()),
            code("code"),
            Span::new(")".to_owned()),
        ]);

        assert_eq!(logical, "(code)");
        assert_eq!(atoms.len(), 3);
        assert!(atoms[1].is_chip());
    }

    #[test]
    fn two_chips_stay_separate_atoms() {
        let (atoms, _) = atomize_text(vec![code("one"), code("two")]);

        assert_eq!(atoms.iter().filter(|atom| atom.is_chip()).count(), 2);
    }

    #[test]
    fn letter_spacing_increases_measured_width() {
        let config = config();
        let plain = Span::new("ab".to_owned()).size(Pixels(10.0));
        let spaced = plain.clone().letter_spacing(Pixels(2.0));
        let plain_width = measure(&[plain], config).min_bounds().width;
        let spaced_width = measure(&[spaced], config).min_bounds().width;

        assert!(spaced_width > plain_width + 1.5);
    }

    #[test]
    fn space_count_sets_the_gap() {
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(
            vec![
                Span::new("a".to_owned()),
                Span::new("   ".to_owned()),
                code("code").padding(Padding::from(4.0)),
            ]
            .into_boxed_slice(),
        );
        let atoms = atomize(&spans, config());
        let space = atoms.iter().find(|atom| atom.is_space()).expect("space");
        let chip = atoms.iter().find(|atom| atom.is_chip()).expect("chip");

        assert!((space.width() - 12.0).abs() < 0.01);
        assert!((chip.content_offset() - 4.0).abs() < 0.01);
    }
}
