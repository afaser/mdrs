use iced::font::{Style, Weight};

use crate::config::FontChain;
use crate::config::colors::ColorRef;
use crate::config::node::Node;

#[derive(Debug, Clone, PartialEq)]
pub struct Role {
    pub font_set: Option<String>,
    pub fonts: FontChain,
    pub size: f32,
    pub weight: Weight,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub color: ColorRef,
    pub ligatures: bool,
}

impl Role {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            font_set: optional_text(node, "font", base.font_set),
            fonts: FontChain::from_node(base.fonts, node.child("fonts")),
            size: number(node, "size", base.size),
            weight: weight(node, "weight", base.weight),
            line_height: number(node, "line-height", base.line_height),
            letter_spacing: number(node, "letter-spacing", base.letter_spacing),
            color: color(node, "color", base.color),
            ligatures: boolean(node, "ligatures", base.ligatures),
        }
    }

    pub fn chain<'a>(
        &'a self,
        fonts: &'a crate::config::fonts::Fonts,
        weight: Weight,
        style: Style,
    ) -> &'a FontChain {
        if let Some(name) = &self.font_set
            && let Some(set) = fonts.set(name)
        {
            let chain = set.chain(weight, style);

            if !chain.names().is_empty() {
                return chain;
            }
        }

        &self.fonts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeadingRole {
    pub size: f32,
    pub weight: Weight,
    pub font_set: Option<String>,
}

impl HeadingRole {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            size: number(node, "size", base.size),
            weight: weight(node, "weight", base.weight),
            font_set: optional_text(node, "font", base.font_set),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub font_set: Option<String>,
    pub fonts: FontChain,
    pub color: ColorRef,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub h1: HeadingRole,
    pub h2: HeadingRole,
    pub h3: HeadingRole,
    pub h4: HeadingRole,
    pub h5: HeadingRole,
    pub h6: HeadingRole,
}

impl Heading {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            font_set: optional_text(node, "font", base.font_set),
            fonts: FontChain::from_node(base.fonts, node.child("fonts")),
            color: color(node, "color", base.color),
            line_height: number(node, "line-height", base.line_height),
            letter_spacing: number(node, "letter-spacing", base.letter_spacing),
            h1: HeadingRole::from_node(base.h1, node.child("h1")),
            h2: HeadingRole::from_node(base.h2, node.child("h2")),
            h3: HeadingRole::from_node(base.h3, node.child("h3")),
            h4: HeadingRole::from_node(base.h4, node.child("h4")),
            h5: HeadingRole::from_node(base.h5, node.child("h5")),
            h6: HeadingRole::from_node(base.h6, node.child("h6")),
        }
    }

    pub fn role(&self, index: usize) -> Role {
        let level = self.level(index);

        Role {
            font_set: level.font_set.clone().or_else(|| self.font_set.clone()),
            fonts: self.fonts.clone(),
            size: level.size,
            weight: level.weight,
            line_height: self.line_height,
            letter_spacing: self.letter_spacing,
            color: self.color,
            ligatures: false,
        }
    }

    fn level(&self, index: usize) -> &HeadingRole {
        match index {
            0 => &self.h1,
            1 => &self.h2,
            2 => &self.h3,
            3 => &self.h4,
            4 => &self.h5,
            _ => &self.h6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Insets {
    pub fn padding(self) -> iced::Padding {
        iced::Padding {
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            left: self.left,
        }
    }

    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            top: number(node, "top", base.top),
            right: number(node, "right", base.right),
            bottom: number(node, "bottom", base.bottom),
            left: number(node, "left", base.left),
        }
    }
}

pub(crate) fn number(node: Node<'_>, name: &str, base: f32) -> f32 {
    node.child(name)
        .and_then(|node| node.number())
        .unwrap_or(base)
}

pub(crate) fn boolean(node: Node<'_>, name: &str, base: bool) -> bool {
    node.child(name)
        .and_then(|node| node.boolean())
        .unwrap_or(base)
}

pub(crate) fn color(node: Node<'_>, name: &str, base: ColorRef) -> ColorRef {
    node.child(name)
        .and_then(|node| node.color())
        .unwrap_or(base)
}

pub(crate) fn optional_text(node: Node<'_>, name: &str, base: Option<String>) -> Option<String> {
    node.child(name)
        .and_then(|node| node.text())
        .map(str::to_owned)
        .or(base)
}

pub(crate) fn optional_number(node: Node<'_>, name: &str, base: Option<f32>) -> Option<f32> {
    node.child(name).and_then(|node| node.number()).or(base)
}

pub(crate) fn optional_weight(node: Node<'_>, name: &str, base: Option<Weight>) -> Option<Weight> {
    node.child(name)
        .and_then(|node| node.text())
        .and_then(parse_weight)
        .or(base)
}

pub(crate) fn weight(node: Node<'_>, name: &str, base: Weight) -> Weight {
    node.child(name)
        .and_then(|node| node.text())
        .and_then(parse_weight)
        .unwrap_or(base)
}

fn parse_weight(value: &str) -> Option<Weight> {
    match value {
        "thin" => Some(Weight::Thin),
        "extra-light" => Some(Weight::ExtraLight),
        "light" => Some(Weight::Light),
        "normal" => Some(Weight::Normal),
        "medium" => Some(Weight::Medium),
        "semibold" => Some(Weight::Semibold),
        "bold" => Some(Weight::Bold),
        "extra-bold" => Some(Weight::ExtraBold),
        "black" => Some(Weight::Black),
        _ => None,
    }
}
