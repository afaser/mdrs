use std::sync::Arc;

use iced::advanced::graphics::text::Paragraph as TextParagraph;
use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Point, Rectangle, Size};

use crate::selection::block_source::BlockSource;
use crate::selection::document_geometry::VisualLine;
use crate::selection::generation::Generation;
use crate::selection::text_layout::{LayoutChange, TextConfig, TextLayout};

#[derive(Debug, Clone, Copy)]
pub(super) struct Press {
    pub(super) link_span: Option<usize>,
}

#[derive(Default)]
pub(super) struct TextState {
    layout: TextLayout,
    press: Option<Press>,
    hovered_span: Option<usize>,
    generation: Generation,
}

impl TextState {
    pub(super) fn sync_generation(&mut self, generation: Generation) {
        if generation == self.generation {
            return;
        }

        self.generation = generation;
        self.reset_interaction();
    }

    pub(super) fn sync_layout(
        &mut self,
        config: TextConfig,
        spans: &Arc<[Span<'static, Uri>]>,
        bounds: Size,
    ) -> bool {
        let change = self.layout.sync(config, spans, bounds);

        if change == LayoutChange::Unchanged {
            return false;
        }

        if change == LayoutChange::Reshaped {
            self.reset_interaction();
        }

        true
    }

    pub(super) fn reset_interaction(&mut self) {
        self.press = None;
        self.hovered_span = None;
    }

    pub(super) fn press(&mut self, press: Press) {
        self.press = Some(press);
        self.hovered_span = None;
    }

    pub(super) fn take_press(&mut self) -> Option<Press> {
        self.press.take()
    }

    pub(super) fn hover(&mut self, link_span: Option<usize>) -> bool {
        if self.hovered_span == link_span {
            return false;
        }

        self.hovered_span = link_span;
        true
    }

    pub(super) fn hovered_span(&self) -> Option<usize> {
        self.hovered_span
    }

    pub(super) fn point_in_text(&self, point: Point) -> bool {
        self.layout.point_in_text(point)
    }

    pub(super) fn logical_at(&self, bounds: Size, point: Point) -> Option<usize> {
        self.layout.logical_at(bounds, point)
    }

    pub(super) fn visual_lines(&self, source: &BlockSource) -> Vec<VisualLine> {
        self.layout.visual_lines(source)
    }

    pub(super) fn hit_link(&self, spans: &[Span<'static, Uri>], point: Point) -> Option<usize> {
        self.layout
            .hit_span(point)
            .filter(|index| spans.get(*index).is_some_and(|span| span.link.is_some()))
    }

    pub(super) fn span_bounds(&self, index: usize) -> Vec<Rectangle> {
        self.layout.span_bounds(index)
    }

    pub(super) fn min_bounds(&self) -> Size {
        self.layout.min_bounds()
    }

    pub(super) fn paragraph(&self) -> &TextParagraph {
        self.layout.paragraph()
    }
}
