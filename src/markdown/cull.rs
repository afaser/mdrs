use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, mouse, overlay, renderer};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

pub struct Cull<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> Cull<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

fn skips(event: &Event, bounds: Rectangle, viewport: &Rectangle) -> bool {
    matches!(event, Event::Mouse(mouse::Event::CursorMoved { .. })) && !bounds.intersects(viewport)
}

fn forwards_interaction(cursor: mouse::Cursor, bounds: Rectangle) -> bool {
    cursor.is_over(bounds)
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Cull<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if skips(event, layout.bounds(), viewport) {
            return;
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if !forwards_interaction(cursor, layout.bounds()) {
            return mouse::Interaction::None;
        }

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<Cull<'a, Message>> for Element<'a, Message> {
    fn from(cull: Cull<'a, Message>) -> Self {
        Element::new(cull)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{Point, Size};

    fn bounds() -> Rectangle {
        Rectangle::new(Point::ORIGIN, Size::new(10.0, 10.0))
    }

    fn moved() -> Event {
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(1.0, 1.0),
        })
    }

    #[test]
    fn cursor_moved_is_skipped_outside_viewport() {
        let viewport = Rectangle::new(Point::new(100.0, 100.0), Size::new(10.0, 10.0));

        assert!(skips(&moved(), bounds(), &viewport));
    }

    #[test]
    fn cursor_moved_reaches_child_inside_viewport() {
        let mut calls = 0;
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0));

        if !skips(&moved(), bounds(), &viewport) {
            calls += 1;
        }

        assert_eq!(calls, 1);
    }

    #[test]
    fn other_events_are_not_skipped() {
        let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let viewport = Rectangle::new(Point::new(100.0, 100.0), Size::new(10.0, 10.0));

        assert!(!skips(&event, bounds(), &viewport));
    }

    #[test]
    fn interaction_is_none_outside_bounds() {
        let inside = mouse::Cursor::Available(Point::new(1.0, 1.0));
        let outside = mouse::Cursor::Available(Point::new(100.0, 100.0));

        assert!(forwards_interaction(inside, bounds()));
        assert!(!forwards_interaction(outside, bounds()));
    }
}
