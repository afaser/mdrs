use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Layout, Shell, layout, overlay, renderer};
use iced::widget::scrollable::AbsoluteOffset;
use iced::{Element, Event, Length, Rectangle, Size, Vector, keyboard, mouse};

use crate::app::Message;
use crate::app::scroll::Scroll;
use crate::app::scroll_offset::Access;
use crate::config::Wheel;
use crate::selection::Generation;

pub struct WheelScroll<'a> {
    content: Element<'a, Message>,
    wheel: Wheel,
    generation: Generation,
    offset: Rc<Cell<f32>>,
}

impl<'a> WheelScroll<'a> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        wheel: Wheel,
        generation: Generation,
        offset: Rc<Cell<f32>>,
    ) -> Self {
        Self {
            content: content.into(),
            wheel,
            generation,
            offset,
        }
    }

    fn sync_scroll(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        let bounds = layout.bounds();
        let Some(content) = layout.children().next() else {
            return;
        };
        let content_bounds = content.bounds();
        let limit = vertical_limit(bounds, content_bounds);
        let mut access = Access::read();

        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut access);

        let offset = access.offset().unwrap_or_default();
        let state: &mut WheelState = tree.state.downcast_mut();

        state.scroll.set_limit(limit);
        state
            .scroll
            .remember(AbsoluteOffset { x: 0.0, y: offset }, limit);
        self.offset.set(offset);
    }

    fn apply_offset(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        y: f32,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            &mut Access::write(y),
        );
        self.offset.set(y);
    }
}

struct WheelState {
    modifiers: keyboard::Modifiers,
    scroll: Scroll,
    generation: Generation,
}

impl WheelState {
    fn new(wheel: Wheel, generation: Generation, offset: f32) -> Self {
        let mut scroll = Scroll::new(wheel);
        scroll.remember(AbsoluteOffset { x: 0.0, y: offset }, 0.0);

        Self {
            modifiers: keyboard::Modifiers::default(),
            scroll,
            generation,
        }
    }

    fn sync(&mut self, wheel: Wheel, generation: Generation, offset: f32) {
        if self.generation == generation {
            return;
        }

        self.generation = generation;
        self.scroll = Scroll::new(wheel);
        self.scroll
            .remember(AbsoluteOffset { x: 0.0, y: offset }, 0.0);
    }
}

impl<'a> Widget<Message, iced::Theme, iced::Renderer> for WheelScroll<'a> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<WheelState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(WheelState::new(
            self.wheel,
            self.generation,
            self.offset.get(),
        ))
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
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let bounds = node.bounds();
        let content_bounds = node.children().first().map(|child| child.bounds());
        let state: &mut WheelState = tree.state.downcast_mut();

        state.sync(self.wheel, self.generation, self.offset.get());

        if let Some(content_bounds) = content_bounds {
            state
                .scroll
                .set_limit(vertical_limit(bounds, content_bounds));
        }

        node
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
        {
            let state: &mut WheelState = tree.state.downcast_mut();
            state.sync(self.wheel, self.generation, self.offset.get());

            if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
                state.modifiers = *modifiers;
            }
        }

        let modifiers = {
            let state: &WheelState = tree.state.downcast_ref();
            state.modifiers
        };

        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event
            && cursor.is_over(layout.bounds())
            && captures_wheel(*delta, modifiers)
        {
            self.sync_scroll(tree, layout, renderer);
            let now = Instant::now();
            let offset = {
                let state: &mut WheelState = tree.state.downcast_mut();
                state.scroll.wheel(*delta, now);
                state.scroll.frame(now)
            };

            if let Some(offset) = offset {
                self.apply_offset(tree, layout, renderer, offset.y);
            }

            shell.capture_event();
            shell.request_redraw();
            return;
        }

        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            let (offset, moving) = {
                let state: &mut WheelState = tree.state.downcast_mut();
                let offset = state.scroll.frame(*now);

                (offset, state.scroll.moving())
            };

            if let Some(offset) = offset {
                self.apply_offset(tree, layout, renderer, offset.y);
            }

            if moving {
                shell.request_redraw();
            }

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
        self.sync_scroll(tree, layout, renderer);
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

fn vertical_limit(bounds: Rectangle, content_bounds: Rectangle) -> f32 {
    (content_bounds.height - bounds.height).max(0.0)
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
    fn generation_change_resets_motion_to_current_offset() {
        let start = Instant::now();
        let mut state = WheelState::new(Wheel::default(), Generation::new(), 0.0);
        state
            .scroll
            .remember(AbsoluteOffset { x: 0.0, y: 0.0 }, 1000.0);
        state
            .scroll
            .wheel(mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 }, start);
        state
            .scroll
            .frame(start + std::time::Duration::from_millis(16));
        let mut generation = Generation::new();
        generation.advance();

        state.sync(Wheel::default(), generation, 320.0);

        assert!(!state.scroll.moving());
        assert_eq!(state.scroll.offset().y, 320.0);
    }

    #[test]
    fn vertical_limit_never_becomes_negative() {
        let empty = Rectangle {
            y: 0.0,
            ..Rectangle::default()
        };
        let content = Rectangle {
            height: 120.0,
            ..Rectangle::default()
        };

        assert_eq!(vertical_limit(empty, content), 120.0);
        assert_eq!(vertical_limit(content, empty), 0.0);
    }

    #[test]
    fn diff_preserves_child_tree() {
        let element: Element<Message> = WheelScroll::new(
            iced::widget::text("content"),
            Wheel::default(),
            Generation::new(),
            Rc::new(Cell::new(0.0)),
        )
        .into();
        let mut tree = Tree::new(&element);

        element.as_widget().diff(&mut tree);

        assert_eq!(tree.children.len(), 1);
    }
}
