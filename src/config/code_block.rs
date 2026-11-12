use crate::config::colors::ColorRef;
use crate::config::frame::Frame;
use crate::config::header::Header;
use crate::config::node::Node;
use crate::config::palette::PaletteKey;
use crate::config::scrollbars::Bar;
use crate::config::{FontChain, Role};

#[derive(Debug, Clone, PartialEq)]
pub struct Content {
    pub padding: f32,
    pub scrollbar: Bar,
}

impl Default for Content {
    fn default() -> Self {
        Self {
            padding: 12.0,
            scrollbar: Bar {
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

impl Content {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            padding: node.number_child("padding").unwrap_or(base.padding),
            scrollbar: Bar::from_node(base.scrollbar, node.child("scrollbar")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub text: Role,
    pub frame: Frame,
    pub header: Header,
    pub content: Content,
}

impl Default for CodeBlock {
    fn default() -> Self {
        let code_fonts = FontChain::new(vec!["code".to_owned()]);

        Self {
            text: Role {
                font_set: None,
                fonts: code_fonts,
                size: 13.0,
                weight: iced::font::Weight::Normal,
                line_height: 1.3,
                letter_spacing: 0.0,
                color: ColorRef::Palette(PaletteKey::Text),
                ligatures: true,
            },
            frame: Frame::default(),
            header: Header::default(),
            content: Content::default(),
        }
    }
}

impl CodeBlock {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            text: Role::from_node(base.text, node.child("text")),
            frame: Frame::from_node(base.frame, node.child("frame")),
            header: Header::from_node(base.header, node.child("header")),
            content: Content::from_node(base.content, node.child("content")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;
    use crate::config::logos::Logos;
    use crate::config::{Easing, IconSource};

    #[test]
    fn deep_copy_color_is_decoded() {
        let document = Document::parse(
            r##"
code-block {
    header {
        copy {
            color "#ff0000"
        }
    }
}
"##,
        )
        .expect("document");
        let code = CodeBlock::from_node(CodeBlock::default(), document.root().child("code-block"));

        assert_eq!(
            code.header.copy.color,
            ColorRef::parse("#ff0000").expect("color")
        );
        assert_eq!(
            code.header.copy.hover_color,
            CodeBlock::default().header.copy.hover_color
        );
    }

    #[test]
    fn header_can_be_hidden() {
        let document = Document::parse(
            r#"
code-block {
    header {
        show #false
    }
}
"#,
        )
        .expect("document");
        let code = CodeBlock::from_node(CodeBlock::default(), document.root().child("code-block"));

        assert!(!code.header.show);
    }

    #[test]
    fn unknown_language_text_is_configurable() {
        let document = Document::parse(
            r#"
code-block {
    header {
        label {
            unknown "Plain"
        }
    }
}
"#,
        )
        .expect("document");
        let code = CodeBlock::from_node(CodeBlock::default(), document.root().child("code-block"));

        assert_eq!(code.header.label.unknown, "Plain");
    }

    #[test]
    fn language_logo_is_selected_with_unknown_fallback() {
        let logos = Logos::default();

        assert_eq!(
            logos.get("rust").icon,
            IconSource::Glyph("\u{e7a8}".to_owned())
        );
        assert_eq!(logos.get("sh").icon, logos.get("bash").icon);
        assert_eq!(
            logos.get("brainfuck").icon,
            IconSource::Glyph("\u{f0f6}".to_owned())
        );
    }

    #[test]
    fn logo_override_is_decoded() {
        let document = Document::parse(
            r##"
code-block {
    header {
        logos {
            rust {
                icon "R"
                color "#ff0000"
            }
            unknown {
                icon "?"
            }
        }
    }
}
"##,
        )
        .expect("document");
        let code = CodeBlock::from_node(CodeBlock::default(), document.root().child("code-block"));

        assert_eq!(
            code.header.logos.get("rust").icon,
            IconSource::Glyph("R".to_owned())
        );
        assert_eq!(
            code.header.logos.get("rust").color,
            ColorRef::parse("#ff0000").expect("color")
        );
        assert_eq!(
            code.header.logos.get("brainfuck").icon,
            IconSource::Glyph("?".to_owned())
        );
    }

    #[test]
    fn unquoted_cubic_bezier_is_decoded() {
        let document = Document::parse(
            r#"
code-block {
    header {
        copy {
            hover-easing 0.25 0.1 0.25 1
        }
    }
}
"#,
        )
        .expect("document");
        let code = CodeBlock::from_node(CodeBlock::default(), document.root().child("code-block"));

        assert_eq!(
            code.header.copy.hover_easing,
            Easing::CubicBezier {
                x1: 0.25,
                y1: 0.1,
                x2: 0.25,
                y2: 1.0,
            }
        );
    }

    #[test]
    fn copy_animation_config_is_decoded() {
        let document = Document::parse(
            r##"
code-block {
    header {
        copy {
            size 22
            radius 10
            check-color "#00ff00"
            check-duration 0.6
            hover-background "#010203"
        }
    }
}
"##,
        )
        .expect("document");
        let code = CodeBlock::from_node(CodeBlock::default(), document.root().child("code-block"));

        assert_eq!(code.header.copy.size, 22.0);
        assert_eq!(code.header.copy.radius, 10.0);
        assert_eq!(
            code.header.copy.check_color,
            ColorRef::parse("#00ff00").expect("color")
        );
        assert_eq!(code.header.copy.check_duration, 0.6);
        assert_eq!(
            code.header.copy.hover_background,
            ColorRef::parse("#010203").expect("color")
        );
    }
}
