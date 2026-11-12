use std::sync::Arc;

use iced::advanced::graphics::text::Paragraph as TextParagraph;
use iced::advanced::graphics::text::cosmic_text::Cursor;
use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Point, Rectangle, Size};

use crate::selection::Selection;
use crate::selection::generation::Generation;
use crate::selection::text_layout::{LayoutChange, TextConfig, TextLayout};

const DRAG_THRESHOLD: f32 = 3.0;

#[derive(Default)]
pub(super) struct TextState {
    layout: TextLayout,
    selection: Selection,
    drag: Option<Drag>,
    hovered_span: Option<usize>,
    generation: Generation,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Drag {
    Pressed {
        origin: Point,
        link_span: Option<usize>,
    },
    Selecting,
}

pub(super) enum Release {
    Click { link_span: Option<usize> },
    Selected { text: Option<String> },
}

impl TextState {
    pub(super) fn sync_generation(&mut self, generation: Generation) {
        if generation == self.generation {
            return;
        }

        self.generation = generation;

        if self.drag.is_some() {
            return;
        }

        self.reset_interaction();
    }

    pub(super) fn sync_layout(
        &mut self,
        config: TextConfig,
        spans: &Arc<[Span<'static, Uri>]>,
        bounds: Size,
    ) {
        let change = self.layout.sync(config, spans, bounds);

        if change != LayoutChange::Reshaped {
            return;
        }

        self.reset_interaction();
    }

    pub(super) fn reset_interaction(&mut self) {
        self.selection.clear();
        self.drag = None;
        self.hovered_span = None;
    }

    pub(super) fn selection_clear(&mut self) {
        self.selection.clear();
    }

    pub(super) fn press(&mut self, point: Point, cursor: Cursor, link_span: Option<usize>) {
        self.selection = Selection::new(cursor, cursor);
        self.drag = Some(Drag::Pressed {
            origin: point,
            link_span,
        });
        self.hovered_span = None;
    }

    pub(super) fn extend(&mut self, point: Point, cursor: Cursor) {
        let Some(drag) = self.drag.as_mut() else {
            return;
        };

        match drag {
            Drag::Pressed { origin, .. } if point.distance(*origin) <= DRAG_THRESHOLD => return,
            Drag::Pressed { .. } => *drag = Drag::Selecting,
            Drag::Selecting => {}
        }

        self.selection.extend(cursor);
        self.hovered_span = None;
    }

    pub(super) fn release(&mut self) -> Option<Release> {
        let drag = self.drag.take()?;

        match drag {
            Drag::Selecting => Some(Release::Selected {
                text: self.selection.text(self.layout.buffer()),
            }),
            Drag::Pressed { link_span, .. } => {
                self.selection.clear();
                Some(Release::Click { link_span })
            }
        }
    }

    pub(super) fn hover(&mut self, link_span: Option<usize>) -> bool {
        if self.hovered_span == link_span {
            return false;
        }

        self.hovered_span = link_span;
        true
    }

    pub(super) fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub(super) fn hovered_span(&self) -> Option<usize> {
        self.hovered_span
    }

    pub(super) fn cursor_at(&self, bounds: Size, point: Point) -> Option<Cursor> {
        self.layout.cursor_at(bounds, point)
    }

    pub(super) fn hit_link(&self, spans: &[Span<'static, Uri>], point: Point) -> Option<usize> {
        self.layout
            .hit_span(point)
            .filter(|index| spans.get(*index).is_some_and(|span| span.link.is_some()))
    }

    pub(super) fn selection_bounds(&self) -> Vec<Rectangle> {
        self.selection.bounds(self.layout.buffer())
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
