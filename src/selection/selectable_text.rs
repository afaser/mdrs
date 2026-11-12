use std::sync::Arc;

use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text::{Alignment, LineHeight, Shaping, Wrapping};
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Layout, Renderer as _, Shell};
use iced::widget::markdown::Uri;
use iced::widget::text::{self as widget_text, Span, Style};
use iced::{
    Background, Color, Element, Event, Font, Length, Pixels, Point, Rectangle, Size, Vector,
};

use crate::selection::decoration::{self, Decoration, LineRule};
use crate::selection::generation::Generation;
use crate::selection::state::{Release, TextState};
use crate::selection::text_layout::TextConfig;

#[derive(Debug, Clone, Copy)]
pub struct TextColors {
    pub text: Color,
    pub selection: Color,
}

pub struct SelectableText<'a, Message> {
    spans: Arc<[Span<'static, Uri>]>,
    size: Pixels,
    font: Font,
    line_height: LineHeight,
    wrapping: Wrapping,
    colors: TextColors,
    generation: Generation,
    decoration: Decoration,
    shaping: Shaping,
    on_link_click: Option<Box<dyn Fn(Uri) -> Message + 'a>>,
    on_selection: Option<Box<dyn Fn(Option<String>) -> Message + 'a>>,
}

impl<'a, Message> SelectableText<'a, Message> {
    pub fn new(
        spans: Arc<[Span<'static, Uri>]>,
        size: Pixels,
        font: Font,
        colors: TextColors,
    ) -> Self {
        Self {
            spans,
            size,
            font,
            line_height: LineHeight::default(),
            wrapping: Wrapping::default(),
            colors,
            generation: Generation::default(),
            decoration: Decoration::default(),
            shaping: Shaping::default(),
            on_link_click: None,
            on_selection: None,
        }
    }

    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.wrapping = wrapping;
        self
    }

    pub fn line_height(mut self, line_height: LineHeight) -> Self {
        self.line_height = line_height;
        self
    }

    pub fn generation(mut self, generation: Generation) -> Self {
        self.generation = generation;
        self
    }

    pub fn decoration(mut self, decoration: Decoration) -> Self {
        self.decoration = decoration;
        self
    }

    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }

    pub fn on_link_click(mut self, on_link_click: impl Fn(Uri) -> Message + 'a) -> Self {
        self.on_link_click = Some(Box::new(on_link_click));
        self
    }

    pub fn on_selection(mut self, on_selection: impl Fn(Option<String>) -> Message + 'a) -> Self {
        self.on_selection = Some(Box::new(on_selection));
        self
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for SelectableText<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TextState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TextState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<TextState>();
        state.sync_generation(self.generation);

        layout::sized(limits, Length::Shrink, Length::Shrink, |limits| {
            let bounds = limits.max();
            let config = TextConfig::new(
                self.size,
                self.font,
                self.line_height,
                self.wrapping,
                self.shaping,
            );
            state.sync_layout(config, &self.spans, bounds);
            state.min_bounds()
        })
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if !layout.bounds().intersects(viewport) {
            return;
        }

        let state = tree.state.downcast_ref::<TextState>();
        let anchor = layout.bounds().anchor(
            state.min_bounds(),
            Alignment::Left,
            iced::alignment::Vertical::Top,
        );
        let translation = anchor - Point::ORIGIN;

        for bounds in state.selection_bounds() {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds + translation,
                    ..renderer::Quad::default()
                },
                Background::Color(self.colors.selection),
            );
        }

        for (index, span) in self.spans.iter().enumerate() {
            let hovered = self.on_link_click.is_some() && Some(index) == state.hovered_span();

            if span.highlight.is_none() && !span.underline && !span.strikethrough && !hovered {
                continue;
            }

            let regions = state.span_bounds(index);

            if let Some(highlight) = span.highlight {
                for bounds in &regions {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle::new(
                                bounds.position()
                                    - Vector::new(span.padding.left, span.padding.top),
                                bounds.size() + Size::new(span.padding.x(), span.padding.y()),
                            ) + translation,
                            border: highlight.border,
                            ..renderer::Quad::default()
                        },
                        highlight.background,
                    );
                }
            }

            if span.underline || span.strikethrough || hovered {
                let size = span.size.unwrap_or(self.size);
                let line_height = span
                    .line_height
                    .unwrap_or(LineHeight::default())
                    .to_absolute(size);
                let color = span.color.unwrap_or(self.colors.text);
                let baseline =
                    translation + Vector::new(0.0, size.0 + (line_height.0 - size.0) / 2.0);

                if span.underline || hovered {
                    decoration::draw(
                        renderer,
                        &regions,
                        baseline,
                        size.0,
                        color,
                        LineRule::Underline,
                        self.decoration,
                    );
                }

                if span.strikethrough {
                    decoration::draw(
                        renderer,
                        &regions,
                        baseline,
                        size.0,
                        color,
                        LineRule::Strikethrough,
                        self.decoration,
                    );
                }
            }
        }

        widget_text::draw(
            renderer,
            defaults,
            layout.bounds(),
            state.paragraph(),
            Style {
                color: Some(self.colors.text),
            },
            viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<TextState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(point) = cursor.position_in(layout.bounds()) else {
                    state.selection_clear();
                    return;
                };

                let Some(text_cursor) = state.cursor_at(layout.bounds().size(), point) else {
                    state.selection_clear();
                    return;
                };

                let link_span = state.hit_link(&self.spans, point);
                state.press(point, text_cursor, link_span);

                if let Some(on_selection) = &self.on_selection {
                    shell.publish(on_selection(None));
                }

                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging() {
                    let Some(point) = relative_point(cursor, layout.bounds()) else {
                        return;
                    };

                    let Some(text_cursor) = state.cursor_at(layout.bounds().size(), point) else {
                        return;
                    };

                    state.extend(point, text_cursor);
                    shell.request_redraw();
                    return;
                }

                let link_span = cursor
                    .position_in(layout.bounds())
                    .and_then(|point| state.hit_link(&self.spans, point));

                if state.hover(link_span) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let Some(release) = state.release() else {
                    return;
                };

                match release {
                    Release::Selected { text } => {
                        if let Some(on_selection) = &self.on_selection {
                            shell.publish(on_selection(text));
                        }
                    }
                    Release::Click { link_span } => {
                        if let Some(on_selection) = &self.on_selection {
                            shell.publish(on_selection(None));
                        }

                        if let (Some(index), Some(on_link_click)) = (link_span, &self.on_link_click)
                            && let Some(link) =
                                self.spans.get(index).and_then(|span| span.link.clone())
                        {
                            shell.publish(on_link_click(link));
                        }
                    }
                }

                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<TextState>();

        match state.hovered_span() {
            Some(_) => mouse::Interaction::Pointer,
            None => mouse::Interaction::Text,
        }
    }
}

fn relative_point(cursor: mouse::Cursor, bounds: Rectangle) -> Option<Point> {
    let position = cursor.position()?;

    Some(Point::new(position.x - bounds.x, position.y - bounds.y))
}

impl<'a, Message> From<SelectableText<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
where
    Message: 'a,
{
    fn from(text: SelectableText<'a, Message>) -> Self {
        Element::new(text)
    }
}
