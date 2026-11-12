use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::markdown::Uri;
use crate::text::Span;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text::{Alignment, LineHeight, Shaping, Wrapping};
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Layout, Renderer as _, Shell};
use iced::{
    Background, Color, Element, Event, Font, Length, Pixels, Point, Rectangle, Size, Vector,
};

use crate::selection::block_source::BlockSource;
use crate::selection::decoration::{self, Decoration, LineRule};
use crate::selection::document_geometry::DocumentGeometry;
use crate::selection::document_selection::DocumentSelection;
use crate::selection::generation::Generation;
use crate::selection::state::{Press, TextState};
use crate::text::TextConfig;

#[derive(Debug, Clone, Copy)]
pub struct TextColors {
    pub text: Color,
    pub selection: Color,
    pub radius: f32,
    pub height: f32,
    pub headings: crate::config::SelectionHeadings,
    pub code_height: f32,
}

impl TextColors {
    pub fn selection_height(&self, source: &BlockSource) -> f32 {
        match (source.is_code(), source.heading_level()) {
            (true, _) => self.code_height,
            (false, Some(level)) => self.headings.get(level),
            (false, None) => self.height,
        }
    }
}

pub struct SelectableText<'a, Message> {
    spans: Arc<[Span<'static>]>,
    size: Pixels,
    font: Font,
    line_height: LineHeight,
    wrapping: Wrapping,
    colors: TextColors,
    generation: Generation,
    decoration: Decoration,
    shaping: Shaping,
    source: Option<BlockSource>,
    selection: Option<Rc<RefCell<DocumentSelection>>>,
    geometry: Option<Rc<RefCell<DocumentGeometry>>>,
    on_link_click: Option<Box<dyn Fn(Uri) -> Message + 'a>>,
    on_selection: Option<Box<dyn Fn(Option<String>) -> Message + 'a>>,
}

impl<'a, Message> SelectableText<'a, Message> {
    pub fn new(spans: Arc<[Span<'static>]>, size: Pixels, font: Font, colors: TextColors) -> Self {
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
            source: None,
            selection: None,
            geometry: None,
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

    fn register(&self, state: &TextState) {
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
            let changed = state.sync_layout(config, &self.spans, bounds);
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

        let state = tree.state.downcast_ref::<TextState>();
        let anchor = layout.bounds().anchor(
            state.min_bounds(),
            Alignment::Left,
            iced::alignment::Vertical::Top,
        );
        let translation = anchor - Point::ORIGIN;

        for (index, span) in self.spans.iter().enumerate() {
            if let Some(highlight) = span.highlight {
                for bounds in state.span_bounds(index) {
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
        }

        for bounds in self.selection_rects() {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds + translation,
                    border: iced::border::rounded(self.colors.radius),
                    ..renderer::Quad::default()
                },
                Background::Color(self.colors.selection),
            );
        }

        for (index, span) in self.spans.iter().enumerate() {
            let hovered = self.on_link_click.is_some() && Some(index) == state.hovered_span();

            if !span.underline && !span.strikethrough && !hovered {
                continue;
            }

            let regions = state.span_bounds(index);
            let size = span.size.unwrap_or(self.size);
            let line_height = span
                .line_height
                .unwrap_or(LineHeight::default())
                .to_absolute(size);
            let color = span.color.unwrap_or(self.colors.text);
            let baseline = translation + Vector::new(0.0, size.0 + (line_height.0 - size.0) / 2.0);

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

        state
            .paragraph()
            .draw(renderer, layout.bounds(), self.colors.text, viewport);
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

        let state = tree.state.downcast_mut::<TextState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(point) = self.local_point(cursor, layout.bounds()) else {
                    return;
                };

                let Some(byte) = state.logical_at(layout.bounds().size(), point) else {
                    return;
                };
                let origin = layout.bounds().position() + Vector::new(point.x, point.y);
                let link_span = state.hit_link(&self.spans, point);
                state.press(Press { link_span });

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

                let link_span = self
                    .local_point(cursor, layout.bounds())
                    .and_then(|point| state.hit_link(&self.spans, point));

                if state.hover(link_span) {
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

                    if let (Some(index), Some(on_link_click)) =
                        (press.link_span, &self.on_link_click)
                        && let Some(link) = self.spans.get(index).and_then(|span| span.link.clone())
                    {
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
        let state = tree.state.downcast_ref::<TextState>();

        if state.hovered_span().is_some() {
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

impl<'a, Message> From<SelectableText<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
where
    Message: 'a,
{
    fn from(text: SelectableText<'a, Message>) -> Self {
        Element::new(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SelectionHeadings;
    use crate::markdown::Article;
    use crate::markdown::Item;

    fn colors() -> TextColors {
        TextColors {
            text: Color::BLACK,
            selection: Color::WHITE,
            radius: 3.0,
            height: 22.0,
            headings: SelectionHeadings::default(),
            code_height: 17.0,
        }
    }

    fn text(article: &Article) -> crate::markdown::Text {
        article
            .items()
            .iter()
            .find_map(|item| match item {
                Item::Paragraph(text) | Item::Heading(_, text) => Some(text.clone()),
                _ => None,
            })
            .expect("text")
    }

    #[test]
    fn selection_height_depends_on_block_kind() {
        let article = Article::parse("# heading\n\nbody");
        let heading = text(&article);
        let block =
            BlockSource::from_text(heading.source().start, article.index().clone(), &heading);
        let colors = colors();

        assert_eq!(colors.selection_height(&block.clone().heading(0)), 44.0);
        assert_eq!(colors.selection_height(&block.clone().heading(2)), 33.0);
        assert_eq!(colors.selection_height(&block), 22.0);
    }
}
