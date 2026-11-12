use iced::Color;

use crate::config::FontChain;
use crate::config::colors::ColorRef;
use crate::config::icon::IconSource;
use crate::config::node::Node;
use crate::config::palette::PaletteKey;

#[derive(Debug, Clone, PartialEq)]
pub struct LogoStyle {
    pub fonts: FontChain,
    pub size: f32,
}

impl Default for LogoStyle {
    fn default() -> Self {
        Self {
            fonts: FontChain::new(vec!["code".to_owned()]),
            size: 16.0,
        }
    }
}

impl LogoStyle {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            fonts: FontChain::from_node(base.fonts, node.child("fonts")),
            size: node.number_child("size").unwrap_or(base.size),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logo {
    pub icon: IconSource,
    pub color: ColorRef,
}

impl Logo {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            icon: IconSource::from_node(node, "icon", "icon-path", base.icon),
            color: node.color_child("color").unwrap_or(base.color),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logos {
    entries: Vec<(String, Logo)>,
    unknown: Logo,
}

impl Logos {
    pub fn get(&self, language: &str) -> &Logo {
        if let Some(logo) = self.find(language) {
            return logo;
        }

        let alias = match language {
            "sh" | "shell" | "zsh" => "bash",
            value => value,
        };

        self.find(alias).unwrap_or(&self.unknown)
    }

    fn find(&self, language: &str) -> Option<&Logo> {
        self.entries
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(language))
            .map(|(_, logo)| logo)
    }

    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        let mut logos = base;

        for child in node.children() {
            let Some(name) = child.name() else {
                continue;
            };

            if name == "unknown" {
                logos.unknown = Logo::from_node(logos.unknown, Some(child));
                continue;
            }

            let current = logos
                .entries
                .iter()
                .find(|(entry, _)| entry.eq_ignore_ascii_case(name))
                .map(|(_, logo)| logo.clone())
                .unwrap_or_else(|| logos.unknown.clone());
            let decoded = Logo::from_node(current, Some(child));

            match logos
                .entries
                .iter_mut()
                .find(|(entry, _)| entry.eq_ignore_ascii_case(name))
            {
                Some((_, logo)) => *logo = decoded,
                None => logos.entries.push((name.to_owned(), decoded)),
            }
        }

        logos
    }
}

impl Default for Logos {
    fn default() -> Self {
        let literal = |red, green, blue| ColorRef::Literal(Color::from_rgb8(red, green, blue));

        Self {
            entries: vec![
                (
                    "rust".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e7a8}".to_owned()),
                        color: literal(0xfa, 0xb3, 0x87),
                    },
                ),
                (
                    "bash".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e795}".to_owned()),
                        color: ColorRef::Palette(PaletteKey::Accent),
                    },
                ),
                (
                    "python".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e73c}".to_owned()),
                        color: literal(0xf9, 0xe2, 0xaf),
                    },
                ),
                (
                    "javascript".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e74e}".to_owned()),
                        color: literal(0xf9, 0xe2, 0xaf),
                    },
                ),
                (
                    "typescript".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e628}".to_owned()),
                        color: literal(0x89, 0xb4, 0xfa),
                    },
                ),
                (
                    "json".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e60b}".to_owned()),
                        color: literal(0xf9, 0xe2, 0xaf),
                    },
                ),
                (
                    "html".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e736}".to_owned()),
                        color: literal(0xfa, 0xb3, 0x87),
                    },
                ),
                (
                    "css".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e749}".to_owned()),
                        color: literal(0x89, 0xb4, 0xfa),
                    },
                ),
                (
                    "go".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e724}".to_owned()),
                        color: literal(0x89, 0xdc, 0xeb),
                    },
                ),
                (
                    "c".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e61e}".to_owned()),
                        color: literal(0x89, 0xb4, 0xfa),
                    },
                ),
                (
                    "cpp".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e61d}".to_owned()),
                        color: literal(0x89, 0xb4, 0xfa),
                    },
                ),
                (
                    "java".to_owned(),
                    Logo {
                        icon: IconSource::Glyph("\u{e738}".to_owned()),
                        color: literal(0xf3, 0x8b, 0xa8),
                    },
                ),
            ],
            unknown: Logo {
                icon: IconSource::Glyph("\u{f0f6}".to_owned()),
                color: ColorRef::Palette(PaletteKey::Text),
            },
        }
    }
}
