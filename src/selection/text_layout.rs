use std::sync::Arc;

use iced::advanced::graphics::text::Paragraph as TextParagraph;
use iced::advanced::graphics::text::cosmic_text::{Buffer, Cursor};
use iced::advanced::text::{self, Alignment, LineHeight, Paragraph as _, Shaping, Text, Wrapping};
use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Font, Pixels, Point, Rectangle, Size};

use crate::selection::block_source::BlockSource;
use crate::selection::document_geometry::{Glyph, VisualLine};

const CURSOR_INSET: f32 = 0.01;

#[derive(Debug, Clone, Copy)]
pub(super) struct TextConfig {
    pub(super) size: Pixels,
    pub(super) font: Font,
    pub(super) line_height: LineHeight,
    pub(super) align_x: Alignment,
    pub(super) align_y: iced::alignment::Vertical,
    pub(super) wrapping: Wrapping,
    pub(super) shaping: Shaping,
}

impl TextConfig {
    pub(super) fn new(
        size: Pixels,
        font: Font,
        line_height: LineHeight,
        wrapping: Wrapping,
        shaping: Shaping,
    ) -> Self {
        Self {
            size,
            font,
            line_height,
            align_x: Alignment::Left,
            align_y: iced::alignment::Vertical::Top,
            wrapping,
            shaping,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LayoutChange {
    Unchanged,
    Resized,
    Reshaped,
}

#[derive(Default)]
pub(super) struct TextLayout {
    paragraph: TextParagraph,
    spans: Option<Arc<[Span<'static, Uri>]>>,
}

impl TextLayout {
    pub(super) fn sync(
        &mut self,
        config: TextConfig,
        spans: &Arc<[Span<'static, Uri>]>,
        bounds: Size,
    ) -> LayoutChange {
        if !self.spans_match(spans) {
            self.rebuild(config, spans, bounds);
            return LayoutChange::Reshaped;
        }

        match self.paragraph.compare(text((), bounds, config)) {
            text::Difference::None => LayoutChange::Unchanged,
            text::Difference::Bounds => {
                self.paragraph.resize(bounds);
                LayoutChange::Resized
            }
            text::Difference::Shape => {
                self.rebuild(config, spans, bounds);
                LayoutChange::Reshaped
            }
        }
    }

    pub(super) fn buffer(&self) -> &Buffer {
        self.paragraph.buffer()
    }

    pub(super) fn paragraph(&self) -> &TextParagraph {
        &self.paragraph
    }

    pub(super) fn min_bounds(&self) -> Size {
        self.paragraph.min_bounds()
    }

    pub(super) fn span_bounds(&self, index: usize) -> Vec<Rectangle> {
        self.paragraph.span_bounds(index)
    }

    pub(super) fn hit_span(&self, point: Point) -> Option<usize> {
        self.paragraph.hit_span(point)
    }

    pub(super) fn visual_lines(&self, source: &BlockSource) -> Vec<VisualLine> {
        let mut starts = Vec::new();
        let mut start = 0;

        for line in &self.buffer().lines {
            starts.push(start);
            start += line.text().len() + 1;
        }

        self.buffer()
            .layout_runs()
            .filter_map(|run| {
                let line_start = *starts.get(run.line_i)?;
                let first = run.glyphs.first()?;
                let last = run.glyphs.last()?;
                let logical = line_start + first.start..line_start + last.end;
                let glyphs = run
                    .glyphs
                    .iter()
                    .map(|glyph| Glyph {
                        x: glyph.x,
                        width: glyph.w,
                        start: line_start + glyph.start,
                        end: line_start + glyph.end,
                    })
                    .collect();

                Some(VisualLine {
                    top: run.line_top,
                    height: run.line_height,
                    left: first.x,
                    source: source.source_range(logical.clone()),
                    logical,
                    glyphs,
                })
            })
            .collect()
    }

    pub(super) fn logical_at(&self, bounds: Size, point: Point) -> Option<usize> {
        let cursor = self.cursor_at(bounds, point)?;
        let mut logical = 0;

        for (index, line) in self.buffer().lines.iter().enumerate() {
            if index == cursor.line {
                return Some(logical + cursor.index.min(line.text().len()));
            }

            logical += line.text().len() + 1;
        }

        None
    }

    pub(super) fn point_in_text(&self, point: Point) -> bool {
        self.buffer().layout_runs().any(|run| {
            point.y >= run.line_top
                && point.y <= run.line_top + run.line_height
                && point.x >= 0.0
                && point.x <= run.line_w
        })
    }

    pub(super) fn cursor_at(&self, bounds: Size, point: Point) -> Option<Cursor> {
        if !has_area(bounds) {
            return None;
        }

        let max_x = (bounds.width - CURSOR_INSET).max(0.0);
        let max_y = (bounds.height - CURSOR_INSET).max(0.0);
        let x = point.x.clamp(0.0, max_x);
        let y = point.y.clamp(0.0, max_y);

        self.buffer().hit(x, y)
    }

    fn spans_match(&self, spans: &Arc<[Span<'static, Uri>]>) -> bool {
        self.spans
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, spans))
    }

    fn rebuild(&mut self, config: TextConfig, spans: &Arc<[Span<'static, Uri>]>, bounds: Size) {
        self.paragraph = TextParagraph::with_spans(text(spans.as_ref(), bounds, config));
        self.spans = Some(Arc::clone(spans));
    }
}

pub(super) fn text<Content>(
    content: Content,
    bounds: Size,
    config: TextConfig,
) -> Text<Content, Font> {
    Text {
        content,
        bounds,
        size: config.size,
        line_height: config.line_height,
        font: config.font,
        align_x: config.align_x,
        align_y: config.align_y,
        shaping: config.shaping,
        wrapping: config.wrapping,
    }
}

fn has_area(bounds: Size) -> bool {
    has_extent(bounds.width) && has_extent(bounds.height)
}

fn has_extent(value: f32) -> bool {
    value > f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::Article;
    use crate::selection::block_source::BlockSource;
    use iced::advanced::text::LineHeight;
    use iced::widget::markdown::Item;
    use iced::widget::text::Span;

    fn config() -> TextConfig {
        crate::theme::isolate_system_fonts();

        TextConfig::new(
            Pixels(16.0),
            Font::default(),
            LineHeight::Relative(1.3),
            Wrapping::default(),
            Shaping::default(),
        )
    }

    fn source(text: &str) -> (Arc<[Span<'static, Uri>]>, BlockSource) {
        let article = Article::parse(text);
        let paragraph = article
            .items()
            .iter()
            .find_map(|item| match item {
                Item::Paragraph(text) => Some(text),
                _ => None,
            })
            .expect("paragraph");
        let block =
            BlockSource::from_text(paragraph.source().start, article.index().clone(), paragraph);
        let spans: Arc<[Span<'static, Uri>]> =
            Arc::from(vec![Span::new(text.to_owned())].into_boxed_slice());

        (spans, block)
    }

    #[test]
    fn layout_reports_unchanged_until_bounds_change() {
        let (spans, _source) = source("short line");
        let mut layout = TextLayout::default();
        let config = config();

        assert_eq!(
            layout.sync(config, &spans, Size::new(400.0, 200.0)),
            LayoutChange::Reshaped
        );
        assert_eq!(
            layout.sync(config, &spans, Size::new(400.0, 200.0)),
            LayoutChange::Unchanged
        );
        assert_eq!(
            layout.sync(config, &spans, Size::new(300.0, 200.0)),
            LayoutChange::Resized
        );
    }

    #[test]
    fn point_in_text_tracks_visual_line_bounds() {
        let (spans, _source) = source("short line");
        let mut layout = TextLayout::default();
        layout.sync(config(), &spans, Size::new(400.0, 200.0));

        assert!(layout.point_in_text(Point::new(1.0, 1.0)));
        assert!(!layout.point_in_text(Point::new(1.0, 1000.0)));
    }

    #[test]
    fn single_line_text_registers_one_visual_line() {
        let (spans, source) = source("short line");
        let mut layout = TextLayout::default();
        layout.sync(config(), &spans, Size::new(400.0, 200.0));
        let lines = layout.visual_lines(&source);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].source, 0..10);
        assert_eq!(lines[0].logical, 0..10);
    }

    #[test]
    fn wrapped_text_registers_several_visual_lines_on_one_source_line() {
        let text = "one two three four five six seven eight nine ten";
        let (spans, source) = source(text);
        let mut layout = TextLayout::default();
        layout.sync(config(), &spans, Size::new(60.0, 200.0));
        let lines = layout.visual_lines(&source);

        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| line.source == (0..text.len())));
    }
}
