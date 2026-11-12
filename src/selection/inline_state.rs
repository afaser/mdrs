use std::sync::Arc;

use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Point, Size};

use crate::selection::block_source::BlockSource;
use crate::selection::document_geometry::VisualLine;
use crate::selection::generation::Generation;
use crate::selection::inline_atom::InlineConfig;
use crate::selection::inline_layout::InlineLayout;
use crate::selection::text_layout::LayoutChange;

#[derive(Debug, Clone)]
pub(super) struct Press {
    pub(super) link: Option<Uri>,
}

#[derive(Default)]
pub(super) struct InlineState {
    layout: InlineLayout,
    press: Option<Press>,
    hovered_link: Option<Uri>,
    generation: Generation,
}

impl InlineState {
    pub(super) fn sync_generation(&mut self, generation: Generation) {
        if generation == self.generation {
            return;
        }

        self.generation = generation;
        self.reset_interaction();
    }

    pub(super) fn sync_layout(
        &mut self,
        spans: &Arc<[Span<'static, Uri>]>,
        config: InlineConfig,
        max_width: f32,
    ) -> bool {
        let change = self.layout.sync(spans, config, max_width);

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
        self.hovered_link = None;
    }

    pub(super) fn press(&mut self, link: Option<Uri>) {
        self.press = Some(Press { link });
        self.hovered_link = None;
    }

    pub(super) fn take_press(&mut self) -> Option<Press> {
        self.press.take()
    }

    pub(super) fn hover(&mut self, link: Option<Uri>) -> bool {
        if self.hovered_link == link {
            return false;
        }

        self.hovered_link = link;
        true
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

    pub(super) fn point_in_text(&self, point: Point) -> bool {
        self.layout.point_in_text(point)
    }

    pub(super) fn visual_lines(&self, source: &BlockSource) -> Vec<VisualLine> {
        self.layout.visual_lines(source)
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
    use iced::advanced::text::{LineHeight, Shaping};
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
    fn layout_change_is_reported_only_for_reflow_or_resize() {
        let spans = spans();
        let config = config();
        let mut state = InlineState::default();

        assert!(state.sync_layout(&spans, config, 10_000.0));
        assert!(!state.sync_layout(&spans, config, 10_000.0));
        assert!(state.sync_layout(&spans, config, 60.0));
    }

    #[test]
    fn layout_reflows_and_keeps_logical_text() {
        let spans = spans();
        let config = config();
        let mut state = InlineState::default();
        state.sync_layout(&spans, config, 10_000.0);
        let wide = state.layout().lines().len();
        state.sync_layout(&spans, config, 60.0);

        assert!(state.layout().lines().len() >= wide);
        assert_eq!(
            state
                .layout()
                .text_between(0, state.layout().logical().len()),
            Some("alpha code omega".to_owned())
        );
    }
}
