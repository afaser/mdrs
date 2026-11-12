use crate::config::colors::ColorRef;
use crate::config::node::Node;
use crate::config::palette::PaletteKey;

#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub track: ColorRef,
    pub thumb: ColorRef,
    pub thumb_hover: ColorRef,
    pub width: f32,
    pub scroller_width: f32,
    pub spacing: f32,
}

impl Bar {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            track: node.color_child("track").unwrap_or(base.track),
            thumb: node.color_child("thumb").unwrap_or(base.thumb),
            thumb_hover: node.color_child("thumb-hover").unwrap_or(base.thumb_hover),
            width: node.number_child("width").unwrap_or(base.width),
            scroller_width: node
                .number_child("scroller-width")
                .unwrap_or(base.scroller_width),
            spacing: node.number_child("spacing").unwrap_or(base.spacing),
        }
    }

    pub fn vertical(&self) -> iced::widget::scrollable::Scrollbar {
        iced::widget::scrollable::Scrollbar::default()
            .width(self.width)
            .scroller_width(self.scroller_width)
            .spacing(self.spacing)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scrollbars {
    pub document: Bar,
    pub code: Bar,
}

impl Default for Scrollbars {
    fn default() -> Self {
        Self {
            document: Bar {
                track: ColorRef::Palette(PaletteKey::Background),
                thumb: ColorRef::Palette(PaletteKey::Border),
                thumb_hover: ColorRef::Palette(PaletteKey::Accent),
                width: 8.0,
                scroller_width: 6.0,
                spacing: 8.0,
            },
            code: Bar {
                track: ColorRef::Palette(PaletteKey::Background),
                thumb: ColorRef::Palette(PaletteKey::Border),
                thumb_hover: ColorRef::Palette(PaletteKey::Accent),
                width: 7.0,
                scroller_width: 7.0,
                spacing: 3.5,
            },
        }
    }
}

impl Scrollbars {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            document: Bar::from_node(base.document, node.child("document")),
            code: Bar::from_node(base.code, node.child("code")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn document_and_code_bars_are_separate() {
        let document = Document::parse(
            r##"
scrollbars {
    document {
        thumb "#ff0000"
    }
}
"##,
        )
        .expect("document");
        let bars =
            Scrollbars::from_node(Scrollbars::default(), document.root().child("scrollbars"));

        assert_eq!(
            bars.document.thumb,
            ColorRef::parse("#ff0000").expect("color")
        );
        assert_eq!(bars.code.thumb, Scrollbars::default().code.thumb);
    }
}
