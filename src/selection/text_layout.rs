use std::sync::Arc;

use iced::advanced::graphics::text::Paragraph as TextParagraph;
use iced::advanced::graphics::text::cosmic_text::{Buffer, Cursor};
use iced::advanced::text::{self, Alignment, LineHeight, Paragraph as _, Shaping, Text, Wrapping};
use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Font, Pixels, Point, Rectangle, Size};

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

    fn spans_match(&self, spans: &[Span<'static, Uri>]) -> bool {
        self.spans.as_deref() == Some(spans)
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
