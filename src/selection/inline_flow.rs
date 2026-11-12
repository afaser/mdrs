use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::markdown::Uri;
use crate::text::Span;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Layout, Renderer as _, Shell};
use iced::{Background, Element, Event, Length, Point, Rectangle, Size, Vector};

use crate::selection::TextColors;
use crate::selection::block_source::BlockSource;
use crate::selection::decoration::Decoration;
use crate::selection::document_geometry::DocumentGeometry;
use crate::selection::document_selection::DocumentSelection;
use crate::selection::generation::Generation;
use crate::selection::inline_atom::InlineConfig;
use crate::selection::inline_state::InlineState;

pub struct InlineFlow<'a, Message> {
    spans: Arc<[Span<'static>]>,
    config: InlineConfig,
    colors: TextColors,
    decoration: Decoration,
    generation: Generation,
    source: Option<BlockSource>,
    selection: Option<Rc<RefCell<DocumentSelection>>>,
    geometry: Option<Rc<RefCell<DocumentGeometry>>>,
    on_link_click: Option<Box<dyn Fn(Uri) -> Message + 'a>>,
    on_selection: Option<Box<dyn Fn(Option<String>) -> Message + 'a>>,
}

impl<'a, Message> InlineFlow<'a, Message> {
    pub fn new(spans: Arc<[Span<'static>]>, config: InlineConfig, colors: TextColors) -> Self {
        Self {
            spans,
            config,
            colors,
            decoration: Decoration::default(),
            generation: Generation::default(),
            source: None,
            selection: None,
            geometry: None,
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

    pub fn source(mut self, source: BlockSource) -> Self {
        self.source = Some(source);
        self
    }

    pub fn selection(mut self, selection: Rc<RefCell<DocumentSelection>>) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn geometry(mut self, geometry: Rc<RefCell<DocumentGeometry>>) -> Self {
        self.geometry = Some(geometry);
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

    fn register(&self, state: &InlineState) {
        let (Some(geometry), Some(source)) = (&self.geometry, &self.source) else {
            return;
        };
        let lines = state.visual_lines(source);

        geometry
            .borrow_mut()
            .register(self.generation, source.clone(), lines);
    }

    fn set_origin(&self, origin: Point) {
        let (Some(geometry), Some(source)) = (&self.geometry, &self.source) else {
            return;
        };

        geometry.borrow_mut().set_origin(source.block(), origin);
    }

    fn selection_rects(&self) -> Vec<Rectangle> {
        let (Some(selection), Some(geometry), Some(source)) =
            (&self.selection, &self.geometry, &self.source)
        else {
            return Vec::new();
        };
        let height = self.colors.selection_height(source);
        let geometry = geometry.borrow();
        let selection = selection.borrow();
        let Some(block) = geometry.block(source.block()) else {
            return Vec::new();
        };

        match selection.line_range() {
            Some(lines) => {
                geometry.line_rects(source.block(), lines, height, geometry.content_width())
            }
            None => match selection.text_range(source.block(), source.logical_len()) {
                Some(logical) => block.text_rects(logical, height),
                None => Vec::new(),
            },
        }
    }

    fn local_point(&self, cursor: mouse::Cursor, bounds: Rectangle) -> Option<Point> {
        cursor.position_in(bounds)
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
            let changed = state.sync_layout(&self.spans, self.config, limits.max().width);
            if changed {
                self.register(state);
            }
            state.min_bounds()
        })
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if !layout.bounds().intersects(viewport) {
            return;
        }

        self.set_origin(layout.bounds().position());

        let state = tree.state.downcast_ref::<InlineState>();
        let origin = layout.bounds().position();
        let lines = state.layout().lines();
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };

        crate::selection::inline_render::draw_chips(renderer, origin, lines, state, &clip);

        let line_selection = self
            .selection
            .as_ref()
            .is_some_and(|selection| selection.borrow().line_range().is_some());
        let selection_clip = match line_selection {
            true => viewport,
            false => &clip,
        };

        for bounds in self.selection_rects() {
            let translated = Rectangle::new(
                bounds.position() + Vector::new(origin.x, origin.y),
                bounds.size(),
            );
            let Some(bounds) = translated.intersection(selection_clip) else {
                continue;
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: iced::border::rounded(self.colors.radius),
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
        self.set_origin(layout.bounds().position());

        let state = tree.state.downcast_mut::<InlineState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(point) = self.local_point(cursor, layout.bounds()) else {
                    return;
                };

                let Some(byte) = state.byte_at(point) else {
                    return;
                };
                let origin = layout.bounds().position() + Vector::new(point.x, point.y);
                let link = state.link_at(point);
                state.press(link.clone());

                if let (Some(selection), Some(source)) = (&self.selection, &self.source) {
                    selection
                        .borrow_mut()
                        .begin_text(source.block(), byte, origin);
                }

                if let Some(on_selection) = &self.on_selection {
                    shell.publish(on_selection(None));
                }

                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.borrow().dragging())
                {
                    return;
                }

                let link = self
                    .local_point(cursor, layout.bounds())
                    .and_then(|point| state.link_at(point));

                if state.hover(link) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let Some(press) = state.take_press() else {
                    return;
                };

                let (dragged, text) = match (&self.selection, &self.geometry) {
                    (Some(selection), Some(geometry)) => {
                        let mut selection = selection.borrow_mut();
                        let dragged = selection.dragged();
                        let text = selection.finish(&geometry.borrow());
                        (dragged, text)
                    }
                    _ => (false, None),
                };

                if dragged {
                    if let Some(on_selection) = &self.on_selection {
                        shell.publish(on_selection(text));
                    }
                } else {
                    if let Some(selection) = &self.selection {
                        selection.borrow_mut().clear();
                    }
                    if let Some(on_selection) = &self.on_selection {
                        shell.publish(on_selection(None));
                    }

                    if let (Some(link), Some(on_link_click)) = (press.link, &self.on_link_click) {
                        shell.publish(on_link_click(link));
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
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<InlineState>();

        if state.hovered_link().is_some() {
            return mouse::Interaction::Pointer;
        }

        let Some(point) = cursor.position_in(layout.bounds()) else {
            return mouse::Interaction::None;
        };

        match state.point_in_text(point) {
            true => mouse::Interaction::Text,
            false => mouse::Interaction::None,
        }
    }
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
