use iced::advanced::Renderer as _;
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Layout, Shell, layout, mouse, overlay, renderer};
use iced::{Background, Border, Color, Element, Event, Length, Rectangle, Size, Vector};

pub(super) fn frame<'a, Message: 'a>(
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
    background: Color,
    border: Border,
) -> Element<'a, Message, iced::Theme, iced::Renderer> {
    Element::new(Frame {
        content: content.into(),
        background,
        border,
    })
}

struct Frame<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
    background: Color,
    border: Border,
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Frame<'_, Message> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: self.border.radius,
                    ..Border::default()
                },
                ..renderer::Quad::default()
            },
            Background::Color(self.background),
        );

        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: self.border,
                ..renderer::Quad::default()
            },
            Background::Color(Color::TRANSPARENT),
        );
    }

    fn operate(
        &mut self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::operation::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message: 'a> From<Frame<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(frame: Frame<'a, Message>) -> Self {
        Element::new(frame)
    }
}
