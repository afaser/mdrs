use iced::advanced::Renderer as _;
use iced::advanced::renderer;
use iced::{Color, Rectangle, Size, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decoration {
    pub underline_offset: f32,
    pub strikethrough_offset: f32,
}

impl Default for Decoration {
    fn default() -> Self {
        Self {
            underline_offset: 0.08,
            strikethrough_offset: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LineRule {
    Underline,
    Strikethrough,
}

impl LineRule {
    fn offset(self, size: f32, decoration: Decoration) -> f32 {
        match self {
            Self::Underline => size * decoration.underline_offset,
            Self::Strikethrough => size * decoration.strikethrough_offset,
        }
    }
}

pub(super) fn draw(
    renderer: &mut iced::Renderer,
    regions: &[Rectangle],
    baseline: Vector,
    size: f32,
    color: Color,
    rule: LineRule,
    decoration: Decoration,
) {
    for bounds in regions {
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle::new(
                    bounds.position() + baseline - Vector::new(0.0, rule.offset(size, decoration)),
                    Size::new(bounds.width, 1.0),
                ),
                ..renderer::Quad::default()
            },
            color,
        );
    }
}
