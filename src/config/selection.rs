use crate::config::colors::ColorRef;
use crate::config::node::Node;
use crate::config::palette::PaletteKey;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection {
    pub color: ColorRef,
    pub underline_offset: f32,
    pub strikethrough_offset: f32,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            color: ColorRef::Palette(PaletteKey::Selection),
            underline_offset: 0.08,
            strikethrough_offset: 0.5,
        }
    }
}

impl Selection {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };
        let decoration = node.child("decoration");

        Self {
            color: node.color_child("color").unwrap_or(base.color),
            underline_offset: decoration
                .and_then(|decoration| decoration.number_child("underline-offset"))
                .unwrap_or(base.underline_offset),
            strikethrough_offset: decoration
                .and_then(|decoration| decoration.number_child("strikethrough-offset"))
                .unwrap_or(base.strikethrough_offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn selection_color_and_offsets_are_decoded() {
        let document = Document::parse(
            r##"
selection {
    color "#ff0000"
    decoration {
        underline-offset 0.1
        strikethrough-offset 0.6
    }
}
"##,
        )
        .expect("document");
        let selection =
            Selection::from_node(Selection::default(), document.root().child("selection"));

        assert_eq!(selection.color, ColorRef::parse("#ff0000").expect("color"));
        assert_eq!(selection.underline_offset, 0.1);
        assert_eq!(selection.strikethrough_offset, 0.6);
    }
}
