use std::sync::Arc;

use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Layout, Renderer as _, Shell};
use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Background, Element, Event, Length, Point, Rectangle, Size, Vector};

use crate::selection::TextColors;
use crate::selection::decoration::Decoration;
use crate::selection::generation::Generation;
use crate::selection::inline_atom::InlineConfig;
use crate::selection::inline_state::{InlineRelease, InlineState};

pub struct InlineFlow<'a, Message> {
    spans: Arc<[Span<'static, Uri>]>,
    config: InlineConfig,
    colors: TextColors,
    decoration: Decoration,
    generation: Generation,
    on_link_click: Option<Box<dyn Fn(Uri) -> Message + 'a>>,
    on_selection: Option<Box<dyn Fn(Option<String>) -> Message + 'a>>,
}

impl<'a, Message> InlineFlow<'a, Message> {
    pub fn new(spans: Arc<[Span<'static, Uri>]>, config: InlineConfig, colors: TextColors) -> Self {
        Self {
            spans,
            config,
            colors,
            decoration: Decoration::default(),
            generation: Generation::default(),
            on_link_click: None,
            on_selection: None,
        }
    }

    pub fn decoration(mut self, decoration: Decoration) -> Self {
        self.decoration = decoration;
        self
    }

    pub fn generation(mut self, generation: Generation) -> Self {
        self.generation = generation;
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

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for InlineFlow<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<InlineState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(InlineState::default())
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
        let state = tree.state.downcast_mut::<InlineState>();
        state.sync_generation(self.generation);

        layout::sized(limits, Length::Shrink, Length::Shrink, |limits| {
            state.sync_layout(&self.spans, self.config, limits.max().width);
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

        let state = tree.state.downcast_ref::<InlineState>();
        let origin = layout.bounds().position();
        let lines = state.layout().lines();
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };

        crate::selection::inline_render::draw_chips(renderer, origin, lines, state, &clip);

        for bounds in state.selection_bounds() {
            let translated = Rectangle::new(
                bounds.position() + Vector::new(origin.x, origin.y),
                bounds.size(),
            );
            let Some(bounds) = translated.intersection(&clip) else {
                continue;
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                Background::Color(self.colors.selection),
            );
        }

        crate::selection::inline_render::draw_decorations(
            renderer,
            origin,
            lines,
            state,
            self.colors.text,
            self.decoration,
            &clip,
        );
        crate::selection::inline_render::draw_text(
            renderer,
            defaults,
            origin,
            lines,
            state,
            self.colors.text,
            &clip,
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
        let state = tree.state.downcast_mut::<InlineState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(point) = cursor.position_in(layout.bounds()) else {
                    state.selection_clear();
                    return;
                };

                let Some(byte) = state.byte_at(point) else {
                    state.selection_clear();
                    return;
                };

                let link = state.link_at(point);
                state.press(point, byte, link);

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

                    state.extend(point);
                    shell.request_redraw();
                    return;
                }

                let link = cursor
                    .position_in(layout.bounds())
                    .and_then(|point| state.link_at(point));

                if state.hover(link) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let Some(release) = state.release() else {
                    return;
                };

                match release {
                    InlineRelease::Selected { text } => {
                        if let Some(on_selection) = &self.on_selection {
                            shell.publish(on_selection(text));
                        }
                    }
                    InlineRelease::Click { link } => {
                        if let Some(on_selection) = &self.on_selection {
                            shell.publish(on_selection(None));
                        }

                        if let (Some(link), Some(on_link_click)) = (link, &self.on_link_click) {
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
        let state = tree.state.downcast_ref::<InlineState>();

        match state.hovered_link() {
            Some(_) => mouse::Interaction::Pointer,
            None => mouse::Interaction::Text,
        }
    }
}

fn relative_point(cursor: mouse::Cursor, bounds: Rectangle) -> Option<Point> {
    let position = cursor.position()?;

    Some(Point::new(position.x - bounds.x, position.y - bounds.y))
}

impl<'a, Message> From<InlineFlow<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
where
    Message: 'a,
{
    fn from(text: InlineFlow<'a, Message>) -> Self {
        Element::new(text)
    }
}
