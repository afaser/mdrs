use std::sync::Arc;

use iced::advanced::graphics::text::Paragraph as TextParagraph;
use iced::advanced::graphics::text::cosmic_text::Cursor;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text::{self, Alignment, LineHeight, Paragraph as _, Shaping, Text, Wrapping};
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::advanced::{Layout, Renderer as _};
use iced::widget::markdown::Uri;
use iced::widget::text::{self as widget_text, Span, Style};
use iced::{
    Background, Color, Element, Event, Font, Length, Pixels, Point, Rectangle, Size, Vector,
};

use crate::selection::Selection;

const DRAG_THRESHOLD: f32 = 3.0;

pub struct SelectableText<'a, Message> {
    spans: Arc<[Span<'static, Uri>]>,
    size: Pixels,
    font: Font,
    line_height: LineHeight,
    align_x: Alignment,
    align_y: iced::alignment::Vertical,
    wrapping: Wrapping,
    width: Length,
    height: Length,
    color: Color,
    selection_color: Color,
    epoch: u64,
    on_link_click: Option<Box<dyn Fn(Uri) -> Message + 'a>>,
    on_selection: Option<Box<dyn Fn(Option<String>) -> Message + 'a>>,
}

impl<'a, Message> SelectableText<'a, Message> {
    pub fn new(spans: Arc<[Span<'static, Uri>]>, size: Pixels, font: Font) -> Self {
        Self {
            spans,
            size,
            font,
            line_height: LineHeight::default(),
            align_x: Alignment::Left,
            align_y: iced::alignment::Vertical::Top,
            wrapping: Wrapping::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            color: Color::BLACK,
            selection_color: Color::from_rgb8(0x45, 0x47, 0x5a),
            epoch: 0,
            on_link_click: None,
            on_selection: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = color;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.wrapping = wrapping;
        self
    }

    pub fn epoch(mut self, epoch: u64) -> Self {
        self.epoch = epoch;
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

#[derive(Default)]
struct State {
    paragraph: TextParagraph,
    spans: Vec<Span<'static, Uri>>,
    selection: Selection,
    drag: Option<Drag>,
    hovered: Option<usize>,
    epoch: u64,
}

struct Drag {
    origin: Point,
    link: Option<usize>,
    moved: bool,
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for SelectableText<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();

        if state.epoch != self.epoch {
            state.selection.clear();
            state.drag = None;
            state.epoch = self.epoch;
        }

        layout::sized(limits, self.width, self.height, |limits| {
            let bounds = limits.max();
            let text = || Text {
                content: self.spans.as_ref(),
                bounds,
                size: self.size,
                line_height: self.line_height,
                font: self.font,
                align_x: self.align_x,
                align_y: self.align_y,
                shaping: Shaping::Advanced,
                wrapping: self.wrapping,
            };

            if state.spans.as_slice() != self.spans.as_ref() {
                state.paragraph = TextParagraph::with_spans(text());
                state.spans = self.spans.as_ref().to_vec();
                state.selection.clear();
                state.drag = None;
            } else {
                match state.paragraph.compare(Text {
                    content: (),
                    bounds,
                    size: self.size,
                    line_height: self.line_height,
                    font: self.font,
                    align_x: self.align_x,
                    align_y: self.align_y,
                    shaping: Shaping::Advanced,
                    wrapping: self.wrapping,
                }) {
                    text::Difference::None => {}
                    text::Difference::Bounds => {
                        state.paragraph.resize(bounds);
                    }
                    text::Difference::Shape => {
                        state.paragraph = TextParagraph::with_spans(text());
                        state.selection.clear();
                        state.drag = None;
                    }
                }
            }

            let _ = renderer;

            state.paragraph.min_bounds()
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

        let state = tree.state.downcast_ref::<State>();
        let anchor =
            layout
                .bounds()
                .anchor(state.paragraph.min_bounds(), self.align_x, self.align_y);
        let translation = anchor - Point::ORIGIN;

        for bounds in state.selection.bounds(state.paragraph.buffer()) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds + translation,
                    ..renderer::Quad::default()
                },
                Background::Color(self.selection_color),
            );
        }

        for (index, span) in self.spans.iter().enumerate() {
            let hovered = self.on_link_click.is_some() && Some(index) == state.hovered;

            if span.highlight.is_none() && !span.underline && !span.strikethrough && !hovered {
                continue;
            }

            let regions = state.paragraph.span_bounds(index);

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
                    .unwrap_or(self.line_height)
                    .to_absolute(size);
                let color = span.color.unwrap_or(self.color);
                let baseline =
                    translation + Vector::new(0.0, size.0 + (line_height.0 - size.0) / 2.0);

                if span.underline || hovered {
                    for bounds in &regions {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle::new(
                                    bounds.position() + baseline - Vector::new(0.0, size.0 * 0.08),
                                    Size::new(bounds.width, 1.0),
                                ),
                                ..renderer::Quad::default()
                            },
                            color,
                        );
                    }
                }

                if span.strikethrough {
                    for bounds in &regions {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle::new(
                                    bounds.position() + baseline - Vector::new(0.0, size.0 / 2.0),
                                    Size::new(bounds.width, 1.0),
                                ),
                                ..renderer::Quad::default()
                            },
                            color,
                        );
                    }
                }
            }
        }

        widget_text::draw(
            renderer,
            defaults,
            layout.bounds(),
            &state.paragraph,
            Style {
                color: Some(self.color),
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
        let state = tree.state.downcast_mut::<State>();
        let inside = cursor.position_in(layout.bounds());
        let position = cursor.position().map(|position| {
            let bounds = layout.bounds().position();

            Point::new(position.x - bounds.x, position.y - bounds.y)
        });

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(point) = inside else {
                    state.selection.clear();
                    return;
                };

                let Some(cursor) = cursor_at(&state.paragraph, layout.bounds().size(), point)
                else {
                    state.selection.clear();
                    return;
                };

                state.selection = Selection::new(cursor, cursor);
                state.drag = Some(Drag {
                    origin: point,
                    link: hovered_link(&state.paragraph, &self.spans, point),
                    moved: false,
                });

                if let Some(on_selection) = &self.on_selection {
                    shell.publish(on_selection(None));
                    state.epoch = self.epoch.wrapping_add(1);
                }

                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(drag) = &mut state.drag {
                    let Some(point) = position else {
                        return;
                    };

                    if !drag.moved && point.distance(drag.origin) > DRAG_THRESHOLD {
                        drag.moved = true;
                    }

                    if drag.moved
                        && let Some(cursor) =
                            cursor_at(&state.paragraph, layout.bounds().size(), point)
                    {
                        state.selection.extend(cursor);
                        state.hovered = None;
                        shell.request_redraw();
                    }
                } else {
                    let hovered =
                        inside.and_then(|point| hovered_link(&state.paragraph, &self.spans, point));

                    if hovered != state.hovered {
                        state.hovered = hovered;
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let Some(drag) = state.drag.take() else {
                    return;
                };

                if drag.moved {
                    if let Some(on_selection) = &self.on_selection {
                        let text = state.selection.text(state.paragraph.buffer());

                        shell.publish(on_selection(text));
                    }
                } else {
                    state.selection.clear();

                    if let Some(on_selection) = &self.on_selection {
                        shell.publish(on_selection(None));
                    }

                    if let (Some(index), Some(on_link_click)) = (drag.link, &self.on_link_click)
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
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if state.hovered.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::Text
        }
    }
}

fn cursor_at(paragraph: &TextParagraph, bounds: Size, point: Point) -> Option<Cursor> {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return None;
    }

    let x = point.x.clamp(0.0, bounds.width - 0.01);
    let y = point.y.clamp(0.0, bounds.height - 0.01);

    paragraph.buffer().hit(x, y)
}

fn hovered_link(
    paragraph: &TextParagraph,
    spans: &[Span<'static, Uri>],
    point: Point,
) -> Option<usize> {
    paragraph
        .hit_span(point)
        .filter(|index| spans.get(*index).is_some_and(|span| span.link.is_some()))
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
