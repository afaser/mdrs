use std::sync::Arc;

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text::{Alignment, LineHeight, Shaping, Wrapping};
use iced::advanced::widget::{Widget, tree};
use iced::{Color, Element, Length, Pixels, Rectangle, Size};

use crate::text::{Difference, Paragraph, Span, TextConfig};

pub struct Label {
    spans: Arc<[Span<'static>]>,
    config: TextConfig,
    color: Option<Color>,
}

impl Label {
    pub fn new(spans: impl Into<Arc<[Span<'static>]>>) -> Self {
        Self {
            spans: spans.into(),
            config: TextConfig::new(
                Pixels(16.0),
                iced::Font::default(),
                LineHeight::default(),
                Wrapping::default(),
                Shaping::default(),
            ),
            color: None,
        }
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.config.size = size.into();
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.config.line_height = line_height.into();
        self
    }

    pub fn font(mut self, font: impl Into<iced::Font>) -> Self {
        self.config.font = font.into();
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn align_x(mut self, alignment: Alignment) -> Self {
        self.config.align_x = alignment;
        self
    }

    pub fn align_y(mut self, alignment: iced::alignment::Vertical) -> Self {
        self.config.align_y = alignment;
        self
    }
}

#[derive(Default)]
struct LabelState {
    paragraph: Paragraph,
    spans: Option<Arc<[Span<'static>]>>,
}

impl LabelState {
    fn sync(
        &mut self,
        spans: &Arc<[Span<'static>]>,
        config: TextConfig,
        bounds: Size,
    ) -> Difference {
        let spans_changed = self
            .spans
            .as_ref()
            .is_none_or(|current| !Arc::ptr_eq(current, spans));

        if spans_changed {
            self.rebuild(spans, config, bounds);
            return Difference::Shape;
        }

        let difference = self.paragraph.compare(config, bounds);

        if difference == Difference::Bounds {
            self.paragraph.resize(bounds);
        }

        if difference == Difference::Shape {
            self.rebuild(spans, config, bounds);
        }

        difference
    }

    fn rebuild(&mut self, spans: &Arc<[Span<'static>]>, config: TextConfig, bounds: Size) {
        self.paragraph = Paragraph::new(spans, config, bounds);
        self.spans = Some(Arc::clone(spans));
    }
}

fn layout_label(
    state: &mut LabelState,
    spans: &Arc<[Span<'static>]>,
    config: TextConfig,
    limits: &layout::Limits,
) -> layout::Node {
    state.sync(spans, config, limits.max());

    layout::sized(limits, Length::Shrink, Length::Shrink, |_| {
        state.paragraph.min_bounds()
    })
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Label
where
    Renderer: iced::advanced::Renderer
        + iced::advanced::graphics::text::Renderer
        + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<LabelState>();

        layout_label(state, &self.spans, self.config, limits)
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if !layout.bounds().intersects(viewport) {
            return;
        }

        let state = tree.state.downcast_ref::<LabelState>();
        let color = self.color.unwrap_or(defaults.text_color);

        state
            .paragraph
            .draw(renderer, layout.bounds(), color, viewport);
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LabelState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(LabelState::default())
    }
}

impl<'a, Message, Theme, Renderer> From<Label> for Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: iced::advanced::Renderer
        + iced::advanced::graphics::text::Renderer
        + iced::advanced::text::Renderer<Font = iced::Font>
        + 'a,
{
    fn from(label: Label) -> Self {
        Element::new(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::layout::Limits;

    fn label() -> Label {
        Label::new(vec![Span::new("label")])
    }

    #[test]
    fn shrink_layout_returns_min_bounds() {
        let label = label();
        let mut state = LabelState::default();
        let node = layout_label(&mut state, &label.spans, label.config, &Limits::NONE);

        assert_eq!(node.size(), state.paragraph.min_bounds());
    }

    #[test]
    fn repeated_layout_with_same_bounds_is_unchanged() {
        let label = label();
        let mut state = LabelState::default();
        let limits = Limits::new(Size::ZERO, Size::new(400.0, 200.0));

        let first = state.sync(&label.spans, label.config, limits.max());
        let second = state.sync(&label.spans, label.config, limits.max());

        assert_eq!(first, Difference::Shape);
        assert_eq!(second, Difference::None);
    }
}
