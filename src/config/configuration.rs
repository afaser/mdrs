use crate::config::Text;
use crate::config::code_block::{CodeBlock, Content};
use crate::config::document::Document;
use crate::config::fonts::Fonts;
use crate::config::highlighting::Highlighting;
use crate::config::motion::Motion;
use crate::config::palette::Palette;
use crate::config::scrollbars::Scrollbars;
use crate::config::selection::Selection;
use crate::config::source::Source;
use crate::config::window::Window;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub window: Window,
    pub palette: Palette,
    pub fonts: Fonts,
    pub text: Text,
    pub code_block: CodeBlock,
    pub scrollbars: Scrollbars,
    pub motion: Motion,
    pub selection: Selection,
    pub highlighting: Highlighting,
}

impl Default for Config {
    fn default() -> Self {
        let scrollbars = Scrollbars::default();
        let code_block = CodeBlock {
            content: Content {
                scrollbar: scrollbars.code.clone(),
                ..CodeBlock::default().content
            },
            ..CodeBlock::default()
        };

        Self {
            window: Window::default(),
            palette: Palette::default(),
            fonts: Fonts::default(),
            text: Text::default(),
            code_block,
            scrollbars,
            motion: Motion::default(),
            selection: Selection::default(),
            highlighting: Highlighting::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let Some(source) = Source::discover() else {
            return Self::default();
        };
        let Some(text) = source.read() else {
            return Self::default();
        };
        let Some(document) = Document::parse(&text) else {
            return Self::default();
        };

        Self::from_document(&document)
    }

    pub fn from_document(document: &Document) -> Self {
        let base = Self::default();
        let root = document.root();
        let theme = root.child("theme");

        let palette =
            Palette::from_node(base.palette, theme.and_then(|theme| theme.child("palette")));
        let fonts = Fonts::from_node(base.fonts, theme.and_then(|theme| theme.child("fonts")));
        let text = Text::from_node(base.text, theme.and_then(|theme| theme.child("text")));
        let scrollbars = Scrollbars::from_node(
            base.scrollbars,
            theme.and_then(|theme| theme.child("scrollbars")),
        );
        let code_seed = CodeBlock {
            content: Content {
                scrollbar: scrollbars.code.clone(),
                ..base.code_block.content
            },
            ..base.code_block
        };
        let code_block =
            CodeBlock::from_node(code_seed, theme.and_then(|theme| theme.child("code-block")));
        let motion = Motion::from_node(base.motion, theme.and_then(|theme| theme.child("motion")));
        let selection = Selection::from_node(
            base.selection,
            theme.and_then(|theme| theme.child("selection")),
        );
        let highlighting = Highlighting::from_node(
            base.highlighting,
            theme.and_then(|theme| theme.child("highlighting")),
        );
        let window = Window::from_node(base.window, root.child("window"));

        Self {
            window,
            palette,
            fonts,
            text,
            code_block,
            scrollbars,
            motion,
            selection,
            highlighting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_document_equals_default() {
        let document = Document::parse("").expect("document");

        assert_eq!(Config::from_document(&document), Config::default());
    }

    #[test]
    fn syntax_error_has_no_document() {
        assert!(Document::parse("theme {").is_none());
    }

    #[test]
    fn unknown_branch_does_not_change_known_values() {
        let document = Document::parse(
            r##"
theme {
    future-branch {
        enabled #true
    }
    palette {
        accent "#ff0000"
    }
}
"##,
        )
        .expect("document");
        let config = Config::from_document(&document);

        assert_eq!(
            config.palette.accent,
            iced::Color::from_rgb8(0xff, 0x00, 0x00)
        );
        assert_eq!(config.palette.background, Palette::default().background);
    }

    #[test]
    fn deep_sections_are_combined() {
        let document = Document::parse(
            r##"
theme {
    code-block {
        header {
            copy {
                color "#ff0000"
            }
        }
    }
    text {
        body {
            size 20
        }
    }
}
"##,
        )
        .expect("document");
        let config = Config::from_document(&document);

        assert_eq!(
            config.code_block.header.copy.color,
            crate::config::colors::ColorRef::parse("#ff0000").expect("color")
        );
        assert_eq!(config.text.body.size, 20.0);
    }
}
