use std::cell::{Cell, RefCell};
use std::rc::Rc;

use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Layout, Shell, layout, overlay, renderer};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse};

use crate::selection::DocumentGeometry;
use crate::selection::DocumentSelection;
use crate::selection::SelectionMode;

pub struct SelectionSurface<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
    selection: Rc<RefCell<DocumentSelection>>,
    geometry: Rc<RefCell<DocumentGeometry>>,
    offset: Rc<Cell<f32>>,
    on_selection: Box<dyn Fn(Option<String>) -> Message + 'a>,
}

impl<'a, Message> SelectionSurface<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
        selection: Rc<RefCell<DocumentSelection>>,
        geometry: Rc<RefCell<DocumentGeometry>>,
        offset: Rc<Cell<f32>>,
        on_selection: impl Fn(Option<String>) -> Message + 'a,
    ) -> Self {
        Self {
            content: content.into(),
            selection,
            geometry,
            offset,
            on_selection: Box::new(on_selection),
        }
    }

    fn content_point(&self, position: Point, bounds: Rectangle) -> Point {
        Point::new(
            position.x - bounds.x,
            position.y - bounds.y + self.offset.get(),
        )
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for SelectionSurface<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
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
        self.geometry
            .borrow_mut()
            .set_content_width(node.bounds().width);

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
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(position) = cursor.position() else {
                    return;
                };
                let point = self.content_point(position, bounds);
                let hit = {
                    let geometry = self.geometry.borrow();
                    geometry
                        .hit_exact(point)
                        .or_else(|| geometry.hit(point).filter(|hit| hit.gutter))
                };
                let Some(hit) = hit else {
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
                    return;
                };

                if !hit.gutter {
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
                    return;
                }

                self.selection.borrow_mut().begin_line(hit.line, point);
                shell.publish((self.on_selection)(None));
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let dragging = self.selection.borrow().dragging();

                if !dragging {
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
                    return;
                }

                let Some(position) = cursor.position() else {
                    return;
                };
                let point = self.content_point(position, bounds);
                let hit = {
                    let geometry = self.geometry.borrow();
                    geometry.hit(point)
                };
                let Some(hit) = hit else {
                    return;
                };

                self.selection.borrow_mut().extend(hit, point);
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let mode = self.selection.borrow().mode();

                if self.selection.borrow().dragging() && mode == SelectionMode::Line {
                    let text = self.selection.borrow_mut().finish(&self.geometry.borrow());

                    shell.publish((self.on_selection)(text));
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
            _ => {}
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
        if let Some(position) = cursor.position() {
            let point = self.content_point(position, layout.bounds());
            let geometry = self.geometry.borrow();

            if point.x <= geometry.gutter_right()
                && geometry.hit_exact(point).is_some_and(|hit| hit.gutter)
            {
                return mouse::Interaction::Text;
            }
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

impl<'a, Message> From<SelectionSurface<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
where
    Message: 'a,
{
    fn from(surface: SelectionSurface<'a, Message>) -> Self {
        Element::new(surface)
    }
}
