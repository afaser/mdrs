use std::sync::Arc;

use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Point, Rectangle, Size};

use crate::selection::generation::Generation;
use crate::selection::inline_atom::InlineConfig;
use crate::selection::inline_layout::InlineLayout;
use crate::selection::text_layout::LayoutChange;

const DRAG_THRESHOLD: f32 = 3.0;

pub(super) enum InlineRelease {
    Click { link: Option<Uri> },
    Selected { text: Option<String> },
}

#[derive(Default)]
pub(super) struct InlineState {
    layout: InlineLayout,
    anchor: Option<usize>,
    focus: Option<usize>,
    drag: Option<Drag>,
    hovered_link: Option<Uri>,
    generation: Generation,
}

#[derive(Debug, Clone)]
enum Drag {
    Pressed { origin: Point, link: Option<Uri> },
    Selecting,
}

impl InlineState {
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
        spans: &Arc<[Span<'static, Uri>]>,
        config: InlineConfig,
        max_width: f32,
    ) {
        if self.layout.sync(spans, config, max_width) != LayoutChange::Reshaped {
            return;
        }

        self.reset_interaction();
    }

    pub(super) fn reset_interaction(&mut self) {
        self.anchor = None;
        self.focus = None;
        self.drag = None;
        self.hovered_link = None;
    }

    pub(super) fn selection_clear(&mut self) {
        self.anchor = None;
        self.focus = None;
    }

    pub(super) fn press(&mut self, point: Point, byte: usize, link: Option<Uri>) {
        self.anchor = Some(byte);
        self.focus = Some(byte);
        self.drag = Some(Drag::Pressed {
            origin: point,
            link,
        });
        self.hovered_link = None;
    }

    pub(super) fn extend(&mut self, point: Point) {
        let Some(drag) = self.drag.as_mut() else {
            return;
        };

        match drag {
            Drag::Pressed { origin, .. } if point.distance(*origin) <= DRAG_THRESHOLD => return,
            Drag::Pressed { .. } => *drag = Drag::Selecting,
            Drag::Selecting => {}
        }

        let byte = self.layout.hit(point).unwrap_or_default();
        self.focus = Some(byte);
        self.hovered_link = None;
    }

    pub(super) fn release(&mut self) -> Option<InlineRelease> {
        let drag = self.drag.take()?;

        match drag {
            Drag::Selecting => Some(InlineRelease::Selected {
                text: self.selected_text(),
            }),
            Drag::Pressed { link, .. } => {
                self.selection_clear();
                Some(InlineRelease::Click { link })
            }
        }
    }

    pub(super) fn hover(&mut self, link: Option<Uri>) -> bool {
        if self.hovered_link == link {
            return false;
        }

        self.hovered_link = link;
        true
    }

    pub(super) fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub(super) fn hovered_link(&self) -> Option<&Uri> {
        self.hovered_link.as_ref()
    }

    pub(super) fn link_at(&self, point: Point) -> Option<Uri> {
        self.layout.link_at(point)
    }

    pub(super) fn byte_at(&self, point: Point) -> Option<usize> {
        self.layout.hit(point)
    }

    pub(super) fn selection_bounds(&self) -> Vec<Rectangle> {
        let (Some(anchor), Some(focus)) = (self.anchor, self.focus) else {
            return Vec::new();
        };

        self.layout.selection_bounds(anchor, focus)
    }

    pub(super) fn selected_text(&self) -> Option<String> {
        let (Some(anchor), Some(focus)) = (self.anchor, self.focus) else {
            return None;
        };

        self.layout.text_between(anchor, focus)
    }

    pub(super) fn min_bounds(&self) -> Size {
        self.layout.size()
    }

    pub(super) fn layout(&self) -> &InlineLayout {
        &self.layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::inline_atom::InlineConfig;
    use iced::advanced::text::{LineHeight, Shaping};
    use iced::widget::markdown::Uri;
    use iced::widget::text::Span;
    use iced::{Color, Font, Padding, Pixels};

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

    fn spans() -> Arc<[Span<'static, Uri>]> {
        Arc::from(
            vec![
                Span::new("alpha ".to_owned()),
                Span::new("code".to_owned())
                    .background(Color::from_rgb8(0x31, 0x32, 0x44))
                    .padding(Padding::from(4.0)),
                Span::new(" omega".to_owned()),
            ]
            .into_boxed_slice(),
        )
    }

    #[test]
    fn selection_survives_resize_and_copies_logical_text() {
        let spans = spans();
        let config = config();
        let mut state = InlineState::default();
        state.sync_layout(&spans, config, 10_000.0);
        let start = state.byte_at(Point::new(0.0, 0.0)).expect("start");
        let end = state.byte_at(Point::new(10_000.0, 0.0)).expect("end");
        state.press(Point::new(0.0, 0.0), start, None);
        state.extend(Point::new(10_000.0, 0.0));
        let selected = state.selected_text();
        let bounds = state.selection_bounds();

        state.sync_layout(&spans, config, 60.0);

        assert_eq!(state.selected_text(), selected);
        assert_eq!(selected.as_deref(), Some("alpha code omega"));
        assert!(!bounds.is_empty());
        assert!(!state.selection_bounds().is_empty());
        assert!(end > start);
    }
}
