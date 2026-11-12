use crate::config::colors::ColorRef;
use crate::config::node::Node;

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub background: ColorRef,
    pub border: ColorRef,
    pub border_width: f32,
    pub border_radius: f32,
    pub margin: f32,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            background: ColorRef::Palette(crate::config::palette::PaletteKey::Surface),
            border: ColorRef::Palette(crate::config::palette::PaletteKey::Border),
            border_width: 1.0,
            border_radius: 12.0,
            margin: 8.0,
        }
    }
}

impl Frame {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };
        let border = node.child("border");

        Self {
            background: node.color_child("background").unwrap_or(base.background),
            border: border
                .and_then(|border| border.color_child("color"))
                .unwrap_or(base.border),
            border_width: border
                .and_then(|border| border.number_child("width"))
                .unwrap_or(base.border_width),
            border_radius: border
                .and_then(|border| border.number_child("radius"))
                .unwrap_or(base.border_radius),
            margin: node.number_child("margin").unwrap_or(base.margin),
        }
    }
}
