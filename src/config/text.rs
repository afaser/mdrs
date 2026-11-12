use iced::font::Weight;

use crate::config::colors::ColorRef;
use crate::config::node::Node;
use crate::config::palette::PaletteKey;
use crate::config::role::{HeadingRole, boolean, color, number, optional_number, optional_weight};
use crate::config::{FontChain, Heading, Insets, Role};

#[derive(Debug, Clone, PartialEq)]
pub struct InlineCode {
    pub role: Role,
    pub background: ColorRef,
    pub padding: Insets,
    pub radius: f32,
}

impl InlineCode {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            role: Role::from_node(base.role, Some(node)),
            background: color(node, "background", base.background),
            padding: Insets::from_node(base.padding, node.child("padding")),
            radius: number(node, "radius", base.radius),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub fonts: Option<FontChain>,
    pub size: Option<f32>,
    pub weight: Option<Weight>,
    pub line_height: Option<f32>,
    pub letter_spacing: Option<f32>,
    pub color: ColorRef,
    pub underline: bool,
}

impl Link {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            fonts: FontChain::optional_from_node(base.fonts, node.child("fonts")),
            size: optional_number(node, "size", base.size),
            weight: optional_weight(node, "weight", base.weight),
            line_height: optional_number(node, "line-height", base.line_height),
            letter_spacing: optional_number(node, "letter-spacing", base.letter_spacing),
            color: color(node, "color", base.color),
            underline: boolean(node, "underline", base.underline),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub body: Role,
    pub heading: Heading,
    pub inline_code: InlineCode,
    pub message: Role,
    pub link: Link,
    pub spacing: f32,
    pub heading_gap: f32,
}

impl Default for Text {
    fn default() -> Self {
        let body_fonts = FontChain::new(vec!["latin".to_owned(), "cyrillic".to_owned()]);
        let code_fonts = FontChain::new(vec!["code".to_owned()]);
        let body = Role {
            font_set: None,
            fonts: body_fonts.clone(),
            size: 17.0,
            weight: Weight::Normal,
            line_height: 1.3,
            letter_spacing: 0.0,
            color: ColorRef::Palette(PaletteKey::Text),
            ligatures: false,
        };
        let heading_role = |size| HeadingRole {
            size,
            weight: Weight::Normal,
            font_set: None,
        };

        Self {
            body: body.clone(),
            heading: Heading {
                font_set: None,
                fonts: body_fonts,
                color: ColorRef::Palette(PaletteKey::Accent),
                line_height: 1.3,
                letter_spacing: 0.0,
                h1: heading_role(34.0),
                h2: heading_role(29.75),
                h3: heading_role(25.5),
                h4: heading_role(21.25),
                h5: heading_role(17.0),
                h6: heading_role(17.0),
            },
            inline_code: InlineCode {
                role: Role {
                    font_set: None,
                    fonts: code_fonts.clone(),
                    size: 13.0,
                    weight: Weight::Normal,
                    line_height: 1.3,
                    letter_spacing: 0.0,
                    color: ColorRef::Palette(PaletteKey::Text),
                    ligatures: false,
                },
                background: ColorRef::Palette(PaletteKey::Surface),
                padding: Insets {
                    top: 4.0,
                    right: 4.0,
                    bottom: 4.0,
                    left: 4.0,
                },
                radius: 4.0,
            },
            message: Role {
                font_set: None,
                fonts: FontChain::new(vec!["latin".to_owned(), "cyrillic".to_owned()]),
                size: 18.0,
                weight: Weight::Normal,
                line_height: 1.3,
                letter_spacing: 0.0,
                color: ColorRef::Palette(PaletteKey::Danger),
                ligatures: false,
            },
            link: Link {
                fonts: None,
                size: None,
                weight: None,
                line_height: None,
                letter_spacing: None,
                color: ColorRef::Palette(PaletteKey::Accent),
                underline: false,
            },
            spacing: 14.875,
            heading_gap: 8.5,
        }
    }
}

impl Text {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            body: Role::from_node(base.body, node.child("body")),
            heading: Heading::from_node(base.heading, node.child("heading")),
            inline_code: InlineCode::from_node(base.inline_code, node.child("inline-code")),
            message: Role::from_node(base.message, node.child("message")),
            link: Link::from_node(base.link, node.child("link")),
            spacing: number(node, "spacing", base.spacing),
            heading_gap: number(node, "heading-gap", base.heading_gap),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;
    use iced::font::Style;

    #[test]
    fn one_role_changes_without_touching_others() {
        let document = Document::parse(
            r#"
text {
    body {
        size 20
    }
}
"#,
        )
        .expect("document");
        let text = Text::from_node(Text::default(), document.root().child("text"));

        assert_eq!(text.body.size, 20.0);
        assert_eq!(text.heading.h1.size, 34.0);
        assert_eq!(text.message.size, 18.0);
    }

    #[test]
    fn heading_color_applies_to_all_levels() {
        let document = Document::parse(
            r##"
text {
    heading {
        color "#ff0000"
        h1 { size 40 }
    }
}
"##,
        )
        .expect("document");
        let text = Text::from_node(Text::default(), document.root().child("text"));

        assert_eq!(
            text.heading.color,
            ColorRef::parse("#ff0000").expect("color")
        );
        assert_eq!(text.heading.h1.size, 40.0);
        assert_eq!(text.heading.h2.size, 29.75);
    }

    #[test]
    fn role_uses_font_set_and_heading_level_override() {
        let document = Document::parse(
            r#"
text {
    body { font "body" }
    heading {
        font "heading"
        h1 { font "code" }
    }
}
"#,
        )
        .expect("document");
        let text = Text::from_node(Text::default(), document.root().child("text"));
        let fonts = crate::config::Fonts::default();

        assert_eq!(
            text.body
                .chain(&fonts, Weight::Normal, Style::Normal)
                .names(),
            ["latin", "cyrillic"]
        );
        assert_eq!(
            text.heading
                .role(1)
                .chain(&fonts, Weight::Normal, Style::Normal)
                .names(),
            ["latin", "cyrillic"]
        );
        assert_eq!(
            text.heading
                .role(0)
                .chain(&fonts, Weight::Normal, Style::Normal)
                .names(),
            ["code"]
        );
    }

    #[test]
    fn link_role_is_decoded() {
        let document = Document::parse(
            r##"
text {
    link {
        fonts "code"
        size 15
        weight "bold"
        line-height 1.5
        color "#ff0000"
        underline #true
    }
}
"##,
        )
        .expect("document");
        let text = Text::from_node(Text::default(), document.root().child("text"));

        assert_eq!(
            text.link.fonts,
            Some(FontChain::new(vec!["code".to_owned()]))
        );
        assert_eq!(text.link.size, Some(15.0));
        assert_eq!(text.link.weight, Some(Weight::Bold));
        assert_eq!(text.link.line_height, Some(1.5));
        assert_eq!(text.link.color, ColorRef::parse("#ff0000").expect("color"));
        assert!(text.link.underline);
    }

    #[test]
    fn inline_code_reads_nested_padding() {
        let document = Document::parse(
            r##"
text {
    inline-code {
        background "#010203"
        padding { left 9 }
    }
}
"##,
        )
        .expect("document");
        let text = Text::from_node(Text::default(), document.root().child("text"));

        assert_eq!(
            text.inline_code.background,
            ColorRef::parse("#010203").expect("color")
        );
        assert_eq!(text.inline_code.padding.left, 9.0);
        assert_eq!(text.inline_code.padding.top, 4.0);
        assert_eq!(text.inline_code.radius, 4.0);
    }

    #[test]
    fn inline_code_padding_and_radius_are_decoded() {
        let document = Document::parse(
            r#"
text {
    inline-code {
        radius 3
        padding { top 1; right 2; bottom 3; left 4 }
    }
}
"#,
        )
        .expect("document");
        let text = Text::from_node(Text::default(), document.root().child("text"));

        assert_eq!(text.inline_code.radius, 3.0);
        assert_eq!(text.inline_code.padding.top, 1.0);
        assert_eq!(text.inline_code.padding.right, 2.0);
        assert_eq!(text.inline_code.padding.bottom, 3.0);
        assert_eq!(text.inline_code.padding.left, 4.0);
    }

    #[test]
    fn role_letter_spacing_is_decoded() {
        let document = Document::parse(
            r#"
text {
    body { letter-spacing 0.5 }
    heading { letter-spacing 1.5 }
    link { letter-spacing 2.5 }
}
"#,
        )
        .expect("document");
        let text = Text::from_node(Text::default(), document.root().child("text"));

        assert_eq!(text.body.letter_spacing, 0.5);
        assert_eq!(text.heading.letter_spacing, 1.5);
        assert_eq!(text.link.letter_spacing, Some(2.5));
        assert_eq!(text.heading.role(0).letter_spacing, 1.5);
    }
}
