use std::ops::Range;
use std::sync::Arc;

use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Point, Size};

use crate::selection::block_source::BlockSource;
use crate::selection::document_geometry::{Glyph, VisualLine};
use crate::selection::inline_atom::{Atom, InlineConfig};
use crate::selection::inline_atomize::{atomize, logical_text};
use crate::selection::text_layout::LayoutChange;

const WIDTH_EPSILON: f32 = 0.01;

#[derive(Debug, Clone)]
pub(super) struct LeadingSpace {
    pub(super) range: Range<usize>,
    pub(super) width: f32,
}

#[derive(Debug, Clone)]
pub(super) struct LineItem {
    pub(super) atom: usize,
    pub(super) range: Range<usize>,
    pub(super) offset: f32,
    pub(super) x: f32,
    pub(super) width: f32,
    pub(super) ascent: f32,
    pub(super) space: Option<LeadingSpace>,
}

#[derive(Debug, Clone)]
pub(super) struct Line {
    pub(super) top: f32,
    pub(super) height: f32,
    pub(super) baseline: f32,
    pub(super) items: Vec<LineItem>,
}

#[derive(Default)]
pub(super) struct InlineLayout {
    atoms: Vec<Atom>,
    logical: String,
    lines: Vec<Line>,
    size: Size,
    spans: Option<Arc<[Span<'static, Uri>]>>,
    config: Option<InlineConfig>,
    width: f32,
}

impl InlineLayout {
    pub(super) fn sync(
        &mut self,
        spans: &Arc<[Span<'static, Uri>]>,
        config: InlineConfig,
        max_width: f32,
    ) -> LayoutChange {
        let spans_changed = self
            .spans
            .as_ref()
            .is_none_or(|current| !Arc::ptr_eq(current, spans));
        let config_changed = self.config != Some(config);

        if spans_changed || config_changed {
            self.atoms = atomize(spans, config);
            self.logical = logical_text(spans);
            self.spans = Some(Arc::clone(spans));
            self.config = Some(config);
            self.width = f32::NAN;
        }

        if spans_changed || max_width.is_nan() || (self.width - max_width).abs() > WIDTH_EPSILON {
            self.reflow(max_width);
            return match spans_changed || config_changed {
                true => LayoutChange::Reshaped,
                false => LayoutChange::Resized,
            };
        }

        LayoutChange::Unchanged
    }

    pub(super) fn atoms(&self) -> &[Atom] {
        &self.atoms
    }

    pub(super) fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub(super) fn point_in_text(&self, point: Point) -> bool {
        self.lines.iter().any(|line| {
            let width = line.items.last().map_or(0.0, |item| item.x + item.width);

            point.y >= line.top
                && point.y <= line.top + line.height
                && point.x >= 0.0
                && point.x <= width
        })
    }

    pub(super) fn size(&self) -> Size {
        self.size
    }
    pub(super) fn visual_lines(&self, source: &BlockSource) -> Vec<VisualLine> {
        self.lines
            .iter()
            .filter_map(|line| {
                let mut glyphs = Vec::new();

                for item in &line.items {
                    if let Some(space) = &item.space {
                        glyphs.push(Glyph {
                            x: item.x - space.width,
                            width: space.width,
                            start: space.range.start,
                            end: space.range.end,
                        });
                    }

                    let atom = &self.atoms[item.atom];
                    glyphs.push(Glyph {
                        x: item.x,
                        width: item.width,
                        start: atom.range.start + item.range.start,
                        end: atom.range.start + item.range.end,
                    });
                }

                let first = glyphs.first()?;
                let last = glyphs.last()?;
                let logical = first.start..last.end;

                Some(VisualLine {
                    top: line.top,
                    height: line.height,
                    left: first.x,
                    source: source.source_range(logical.clone()),
                    logical,
                    glyphs,
                })
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn logical(&self) -> &str {
        &self.logical
    }

    #[cfg(test)]
    pub(super) fn text_between(&self, from: usize, to: usize) -> Option<String> {
        let from = from.min(to).min(self.logical.len());
        let to = to.max(from).min(self.logical.len());

        self.logical.get(from..to).map(str::to_owned)
    }

    pub(super) fn hit(&self, point: Point) -> Option<usize> {
        if self.lines.is_empty() {
            return Some(0);
        }

        let line = self.line_at(point.y);
        let Some(item) = line.items.iter().find(|item| point.x < item.x + item.width) else {
            return Some(line.items.last()?.atom_end(self));
        };
        let atom = &self.atoms[item.atom];

        if point.x <= item.x {
            return Some(atom.range.start + item.range.start);
        }

        if point.x >= item.x + item.width {
            return Some(atom.range.start + item.range.end);
        }

        let x = point.x - item.x + item.offset - atom.content_offset();
        let y = point.y - (line.baseline - atom.content_ascent());
        let Some(paragraph) = atom.paragraph() else {
            return Some(atom.range.start + item.range.start);
        };
        let Some(cursor) = paragraph.buffer().hit(x, y) else {
            return Some(atom.range.start + item.range.start);
        };
        let local = match cursor.line {
            0 => cursor.index.clamp(item.range.start, item.range.end),
            _ => item.range.end,
        };

        Some(atom.range.start + local)
    }

    pub(super) fn link_at(&self, point: Point) -> Option<Uri> {
        let byte = self.hit(point)?;

        for atom in &self.atoms {
            if !atom.range.contains(&byte) {
                continue;
            }

            let text = atom.text()?;
            let local = byte - atom.range.start;

            return text
                .styles()
                .iter()
                .find(|style| style.range.contains(&local))
                .and_then(|style| style.link.clone());
        }

        None
    }

    fn reflow(&mut self, max_width: f32) {
        let mut lines = Vec::new();
        let mut items: Vec<LineItem> = Vec::new();
        let mut line_width: f32 = 0.0;
        let mut ascent: f32 = 0.0;
        let mut descent: f32 = 0.0;
        let mut pending_space: f32 = 0.0;
        let mut pending_space_range: Option<Range<usize>> = None;
        let mut top: f32 = 0.0;
        let mut size_width: f32 = 0.0;

        for (index, atom) in self.atoms.iter().enumerate() {
            if atom.is_space() {
                if !items.is_empty() {
                    pending_space += atom.width();
                    pending_space_range = Some(match pending_space_range {
                        Some(mut range) => {
                            range.end = atom.range.end;
                            range
                        }
                        None => atom.range.clone(),
                    });
                }
                continue;
            }

            let advance = pending_space;
            let candidate = line_width + advance + atom.width();

            if !items.is_empty() && candidate > max_width + WIDTH_EPSILON {
                finish_line(
                    &mut lines,
                    &mut items,
                    &mut top,
                    &mut size_width,
                    ascent,
                    descent,
                );
                line_width = 0.0;
                ascent = 0.0;
                descent = 0.0;
                pending_space = 0.0;
                pending_space_range = None;
            }

            if let Some(fragments) = atom.fragments(max_width) {
                if !items.is_empty() {
                    finish_line(
                        &mut lines,
                        &mut items,
                        &mut top,
                        &mut size_width,
                        ascent,
                        descent,
                    );
                    line_width = 0.0;
                    ascent = 0.0;
                    descent = 0.0;
                }

                for fragment in fragments {
                    items.push(LineItem {
                        atom: index,
                        range: fragment.range,
                        offset: fragment.offset,
                        x: 0.0,
                        width: fragment.width,
                        ascent: atom.ascent(),
                        space: None,
                    });
                    ascent = ascent.max(atom.ascent());
                    descent = descent.max(atom.descent());
                    finish_line(
                        &mut lines,
                        &mut items,
                        &mut top,
                        &mut size_width,
                        ascent,
                        descent,
                    );
                    ascent = 0.0;
                    descent = 0.0;
                }

                pending_space = 0.0;
                continue;
            }

            line_width += if items.is_empty() { 0.0 } else { pending_space };
            items.push(LineItem {
                atom: index,
                range: 0..atom.content_len(),
                offset: 0.0,
                x: line_width,
                width: atom.width(),
                ascent: atom.ascent(),
                space: pending_space_range.take().map(|range| LeadingSpace {
                    range,
                    width: pending_space,
                }),
            });
            line_width += atom.width();
            ascent = ascent.max(atom.ascent());
            descent = descent.max(atom.descent());
            pending_space = 0.0;
        }

        finish_line(
            &mut lines,
            &mut items,
            &mut top,
            &mut size_width,
            ascent,
            descent,
        );

        self.lines = lines;
        self.size = Size::new(size_width, top);
        self.width = max_width;
    }

    fn line_at(&self, y: f32) -> &Line {
        self.lines
            .iter()
            .find(|line| y < line.top + line.height)
            .unwrap_or_else(|| &self.lines[self.lines.len() - 1])
    }
}

impl LineItem {
    fn atom_end(&self, layout: &InlineLayout) -> usize {
        layout.atoms[self.atom].range.end
    }
}

fn finish_line(
    lines: &mut Vec<Line>,
    items: &mut Vec<LineItem>,
    top: &mut f32,
    size_width: &mut f32,
    ascent: f32,
    descent: f32,
) {
    if items.is_empty() {
        return;
    }

    let height = ascent + descent;
    let width = items
        .last()
        .map(|item| item.x + item.width)
        .unwrap_or_default();
    *size_width = size_width.max(width);
    lines.push(Line {
        top: *top,
        height,
        baseline: *top + ascent,
        items: std::mem::take(items),
    });
    *top += height;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FontChain, Fonts};
    use crate::markdown::Article;
    use crate::selection::block_source::BlockSource;
    use crate::selection::inline_atom::InlineConfig;
    use crate::theme::FontBook;
    use iced::advanced::text::{LineHeight, Shaping};
    use iced::widget::markdown::Item;
    use iced::{Font, Padding, Pixels};

    fn font() -> Font {
        crate::theme::isolate_system_fonts();
        let book = FontBook::new(&Fonts::default());

        book.chain_font(
            &FontChain::new(vec!["latin".to_owned()]),
            iced::font::Weight::Normal,
            iced::font::Style::Normal,
        )
        .expect("font")
    }

    fn config() -> InlineConfig {
        InlineConfig {
            size: Pixels(17.0),
            font: font(),
            line_height: LineHeight::Relative(1.3),
            shaping: Shaping::Advanced,
            space: 4.0,
        }
    }

    fn code_span(text: &str) -> Span<'static, Uri> {
        Span::new(text.to_owned())
            .background(iced::Color::from_rgb8(0x31, 0x32, 0x44))
            .padding(Padding::from(4.0))
    }

    #[test]
    fn chip_does_not_fit_remaining_line_goes_to_next_line() {
        let spans: Arc<[Span<'static, Uri>]> =
            Arc::from(vec![Span::new("alpha ".to_owned()), code_span("code")].into_boxed_slice());
        let mut layout = InlineLayout::default();
        let config = config();
        layout.sync(&spans, config, 10_000.0);

        let first_width = layout
            .lines()
            .first()
            .and_then(|line| line.items.first())
            .map(|item| item.width)
            .unwrap_or_default();
        let chip_width = layout
            .atoms()
            .iter()
            .find(|atom| atom.is_chip())
            .map(Atom::width)
            .unwrap_or_default();
        layout.sync(&spans, config, first_width + chip_width - 1.0);

        assert_eq!(layout.lines().len(), 2);
        assert!(
            layout.lines()[0]
                .items
                .iter()
                .all(|item| !layout.atoms()[item.atom].is_chip())
        );
    }

    #[test]
    fn resize_reflows_lines_without_splitting_chip() {
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(
            vec![
                Span::new("one two ".to_owned()),
                code_span("chip"),
                Span::new(" three".to_owned()),
            ]
            .into_boxed_slice(),
        );
        let mut layout = InlineLayout::default();
        let config = config();
        layout.sync(&spans, config, 10_000.0);
        let wide = layout.lines().len();
        layout.sync(&spans, config, 80.0);
        let narrow = layout.lines().len();

        assert!(narrow >= wide);
        assert_eq!(
            layout
                .lines()
                .iter()
                .flat_map(|line| line.items.iter())
                .filter(|item| layout.atoms()[item.atom].is_chip())
                .count(),
            1
        );
    }

    #[test]
    fn chip_padding_increases_line_height() {
        let spans: Arc<[Span<'static, Uri>]> =
            Arc::from(vec![code_span("chip")].into_boxed_slice());
        let mut layout = InlineLayout::default();
        let config = config();
        layout.sync(&spans, config, 10_000.0);
        let chip = layout
            .atoms()
            .iter()
            .find(|atom| atom.is_chip())
            .expect("chip");
        let line = &layout.lines()[0];

        assert!(line.height >= chip.content_ascent() + chip.content_descent() + 8.0);
    }

    #[test]
    fn selection_across_chip_copies_logical_text_with_spaces() {
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(
            vec![
                Span::new("alpha beta ".to_owned()),
                code_span("code"),
                Span::new(" omega".to_owned()),
            ]
            .into_boxed_slice(),
        );
        let mut layout = InlineLayout::default();
        layout.sync(&spans, config(), 10_000.0);
        let text = layout.text_between(0, layout.logical().len());

        assert_eq!(text.as_deref(), Some("alpha beta code omega"));
    }

    #[test]
    fn hit_inside_chip_maps_to_code_byte() {
        let spans: Arc<[Span<'static, Uri>]> =
            Arc::from(vec![code_span("code")].into_boxed_slice());
        let mut layout = InlineLayout::default();
        layout.sync(&spans, config(), 10_000.0);
        let line = &layout.lines()[0];
        let item = &line.items[0];
        let atom = &layout.atoms()[item.atom];
        let point = Point::new(
            item.x + atom.content_offset() + atom.content_width() * 0.5,
            line.baseline - atom.content_ascent() + 1.0,
        );
        let byte = layout.hit(point).expect("byte");

        assert!(byte > atom.range.start);
        assert!(byte <= atom.range.end);
    }

    #[test]
    fn visual_lines_cover_logical_text_and_source_line() {
        let source_text = "alpha code omega";
        let article = Article::parse(source_text);
        let paragraph = article
            .items()
            .iter()
            .find_map(|item| match item {
                Item::Paragraph(text) => Some(text),
                _ => None,
            })
            .expect("paragraph");
        let source =
            BlockSource::from_text(paragraph.source().start, article.index().clone(), paragraph);
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(
            vec![
                Span::new("alpha ".to_owned()),
                code_span("code"),
                Span::new(" omega".to_owned()),
            ]
            .into_boxed_slice(),
        );
        let mut layout = InlineLayout::default();
        layout.sync(&spans, config(), 10_000.0);
        let lines = layout.visual_lines(&source);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].logical, 0..16);
        assert_eq!(lines[0].source, 0..source_text.len());
    }

    #[test]
    fn point_in_text_is_false_outside_visual_lines() {
        let spans: Arc<[Span<'static, Uri>]> =
            Arc::from(vec![Span::new("alpha ".to_owned()), code_span("code")].into_boxed_slice());
        let mut layout = InlineLayout::default();
        layout.sync(&spans, config(), 10_000.0);

        assert!(layout.point_in_text(Point::new(1.0, 1.0)));
        assert!(!layout.point_in_text(Point::new(1.0, 10_000.0)));
    }

    #[test]
    fn unchanged_layout_is_not_rebuilt() {
        let spans: Arc<[Span<'static, Uri>]> =
            Arc::from(vec![Span::new("alpha ".to_owned()), code_span("code")].into_boxed_slice());
        let mut layout = InlineLayout::default();
        let first = layout.sync(&spans, config(), 200.0);
        let same = layout.sync(&spans, config(), 200.0);
        let resized = layout.sync(&spans, config(), 100.0);

        assert_eq!(first, LayoutChange::Reshaped);
        assert_eq!(same, LayoutChange::Unchanged);
        assert_eq!(resized, LayoutChange::Resized);
    }

    #[test]
    fn shrink_width_does_not_exceed_available_width() {
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(
            vec![
                Span::new("alpha ".to_owned()),
                code_span("code"),
                Span::new(" omega".to_owned()),
            ]
            .into_boxed_slice(),
        );
        let mut layout = InlineLayout::default();
        layout.sync(&spans, config(), 10_000.0);
        let chip_width = layout
            .atoms()
            .iter()
            .find(|atom| atom.is_chip())
            .map(Atom::width)
            .expect("chip");
        let available = chip_width + 10.0;
        layout.sync(&spans, config(), available);

        assert!(
            layout.size().width <= available + WIDTH_EPSILON,
            "width={}",
            layout.size().width
        );
        assert!(layout.size().height > 0.0);
    }

    #[test]
    fn long_text_atom_continues_across_lines() {
        let text = "supercalifragilisticexpialidocious";
        let spans: Arc<[Span<'static, Uri>]> =
            Arc::from(vec![Span::new(text.to_owned())].into_boxed_slice());
        let mut layout = InlineLayout::default();
        layout.sync(&spans, config(), 50.0);

        assert!(layout.lines().len() > 1);
        assert!(layout.lines().iter().all(|line| {
            line.items
                .iter()
                .all(|item| item.width <= 50.0 + WIDTH_EPSILON)
        }));
        assert_eq!(
            layout
                .lines()
                .iter()
                .flat_map(|line| line.items.iter())
                .map(|item| item.range.len())
                .sum::<usize>(),
            text.len()
        );
    }
}
