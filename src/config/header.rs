use crate::config::colors::ColorRef;
use crate::config::copy_button::CopyButton;
use crate::config::logos::{LogoStyle, Logos};
use crate::config::node::Node;
use crate::config::{FontChain, Insets};

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub fonts: FontChain,
    pub size: f32,
    pub color: ColorRef,
    pub capitalize: bool,
    pub unknown: String,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            fonts: FontChain::new(vec!["code".to_owned()]),
            size: 13.0,
            color: ColorRef::Palette(crate::config::palette::PaletteKey::Text),
            capitalize: true,
            unknown: "Text".to_owned(),
        }
    }
}

impl Label {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            fonts: FontChain::from_node(base.fonts, node.child("fonts")),
            size: node.number_child("size").unwrap_or(base.size),
            color: node.color_child("color").unwrap_or(base.color),
            capitalize: node.boolean_child("capitalize").unwrap_or(base.capitalize),
            unknown: node
                .text_child("unknown")
                .map(str::to_owned)
                .unwrap_or(base.unknown),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub show: bool,
    pub height: f32,
    pub background: ColorRef,
    pub padding: Insets,
    pub logo: LogoStyle,
    pub logos: Logos,
    pub label: Label,
    pub copy: CopyButton,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            show: true,
            height: 32.0,
            background: ColorRef::Palette(crate::config::palette::PaletteKey::Background),
            padding: Insets {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 12.0,
            },
            logo: LogoStyle::default(),
            logos: Logos::default(),
            label: Label::default(),
            copy: CopyButton::default(),
        }
    }
}

impl Header {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            show: node.boolean_child("show").unwrap_or(base.show),
            height: node.number_child("height").unwrap_or(base.height),
            background: node.color_child("background").unwrap_or(base.background),
            padding: Insets::from_node(base.padding, node.child("padding")),
            logo: LogoStyle::from_node(base.logo, node.child("logo")),
            logos: Logos::from_node(base.logos, node.child("logos")),
            label: Label::from_node(base.label, node.child("label")),
            copy: CopyButton::from_node(base.copy, node.child("copy")),
        }
    }
}
