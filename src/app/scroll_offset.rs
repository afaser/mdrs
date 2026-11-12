use iced::advanced::widget::{Id, Operation, operation};
use iced::widget::scrollable::AbsoluteOffset;
use iced::{Rectangle, Vector};

enum Mode {
    Read,
    Write(f32),
}

pub(super) struct Access {
    mode: Mode,
    offset: Option<f32>,
}

impl Access {
    pub(super) fn read() -> Self {
        Self {
            mode: Mode::Read,
            offset: None,
        }
    }

    pub(super) fn write(y: f32) -> Self {
        Self {
            mode: Mode::Write(y),
            offset: None,
        }
    }

    pub(super) fn offset(&self) -> Option<f32> {
        self.offset
    }
}

impl Operation for Access {
    fn traverse(&mut self, _operate: &mut dyn FnMut(&mut dyn Operation)) {}

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn operation::Scrollable,
    ) {
        match self.mode {
            Mode::Read => self.offset = Some(translation.y),
            Mode::Write(y) => state.scroll_to(AbsoluteOffset {
                x: None,
                y: Some(y),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::scrollable::RelativeOffset;

    #[derive(Default)]
    struct Scroll {
        target: Option<AbsoluteOffset<Option<f32>>>,
    }

    impl operation::Scrollable for Scroll {
        fn snap_to(&mut self, _offset: RelativeOffset<Option<f32>>) {}

        fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
            self.target = Some(offset);
        }

        fn scroll_by(
            &mut self,
            _offset: AbsoluteOffset,
            _bounds: Rectangle,
            _content_bounds: Rectangle,
        ) {
        }
    }

    fn translation(y: f32) -> Vector {
        Vector::new(0.0, y)
    }

    #[test]
    fn read_returns_vertical_translation() {
        let mut access = Access::read();
        let mut scroll = Scroll::default();

        access.scrollable(
            None,
            Rectangle::default(),
            Rectangle::default(),
            translation(42.0),
            &mut scroll,
        );

        assert_eq!(access.offset(), Some(42.0));
    }

    #[test]
    fn write_scrolls_to_absolute_offset() {
        let mut access = Access::write(37.0);
        let mut scroll = Scroll::default();

        access.scrollable(
            None,
            Rectangle::default(),
            Rectangle::default(),
            translation(0.0),
            &mut scroll,
        );

        assert_eq!(
            scroll.target,
            Some(AbsoluteOffset {
                x: None,
                y: Some(37.0)
            })
        );
    }
}
