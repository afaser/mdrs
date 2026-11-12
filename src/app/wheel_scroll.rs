use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Layout, Shell, layout, overlay, renderer};
use iced::{Element, Event, Length, Rectangle, Size, Vector, keyboard, mouse};

use crate::app::Message;

pub struct WheelScroll<'a> {
    content: Element<'a, Message>,
}

impl<'a> WheelScroll<'a> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

#[derive(Default)]
struct WheelState {
    modifiers: keyboard::Modifiers,
}

impl<'a> Widget<Message, iced::Theme, iced::Renderer> for WheelScroll<'a> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<WheelState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(WheelState::default())
    }

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
        let state: &mut WheelState = tree.state.downcast_mut();

        if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            state.modifiers = *modifiers;
        }

        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event
            && cursor.is_over(layout.bounds())
            && captures_wheel(*delta, state.modifiers)
        {
            shell.publish(Message::Wheeled(*delta));
            shell.capture_event();
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

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
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

fn wheel_vertical(delta: mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => y,
    }
}

fn captures_wheel(delta: mouse::ScrollDelta, modifiers: keyboard::Modifiers) -> bool {
    let vertical = wheel_vertical(delta);

    !is_zero(vertical) && !modifiers.shift()
}

fn is_zero(value: f32) -> bool {
    value.abs() <= f32::EPSILON
}

impl<'a> From<WheelScroll<'a>> for Element<'a, Message> {
    fn from(scroll: WheelScroll<'a>) -> Self {
        Element::new(scroll)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_capture_depends_on_direction_and_shift() {
        let none = keyboard::Modifiers::default();
        let shift = keyboard::Modifiers::SHIFT;

        assert!(captures_wheel(
            mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
            none
        ));
        assert!(captures_wheel(
            mouse::ScrollDelta::Pixels { x: 0.0, y: -15.0 },
            none
        ));
        assert!(!captures_wheel(
            mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
            shift
        ));
        assert!(!captures_wheel(
            mouse::ScrollDelta::Lines { x: 1.0, y: 0.0 },
            none
        ));
        assert!(!captures_wheel(
            mouse::ScrollDelta::Pixels { x: 15.0, y: 0.0 },
            none
        ));
        assert!(!captures_wheel(
            mouse::ScrollDelta::Lines { x: 0.0, y: 0.0 },
            none
        ));
    }

    #[test]
    fn diff_preserves_child_tree() {
        let element: Element<Message> = WheelScroll::new(iced::widget::text("content")).into();
        let mut tree = Tree::new(&element);

        element.as_widget().diff(&mut tree);

        assert_eq!(tree.children.len(), 1);
    }
}
