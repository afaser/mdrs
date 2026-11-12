use crate::config::face::{EmbeddedFace, Face, Faces};
use crate::config::node::Node;
use crate::config::{FontSet, FontSets};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Latin,
    Cyrillic,
    Common,
    Any,
}

impl Script {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "latin" => Some(Self::Latin),
            "cyrillic" => Some(Self::Cyrillic),
            "common" => Some(Self::Common),
            "any" => Some(Self::Any),
            _ => None,
        }
    }

    pub fn of(character: char) -> Self {
        if is_cyrillic(character) {
            return Self::Cyrillic;
        }

        if character.is_ascii_alphabetic() || is_latin(character) {
            return Self::Latin;
        }

        Self::Common
    }
}

fn is_cyrillic(character: char) -> bool {
    matches!(
        character as u32,
        0x0400..=0x052f | 0x2de0..=0x2dff | 0xa640..=0xa69f
    )
}

fn is_latin(character: char) -> bool {
    matches!(character as u32, 0x00c0..=0x024f | 0x1e00..=0x1eff)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedFont {
    pub name: String,
    pub family: String,
    pub script: Script,
    pub faces: Faces,
    pub ligatures: bool,
}

impl NamedFont {
    fn builtin(name: &str, family: &str, script: Script, faces: Faces, ligatures: bool) -> Self {
        Self {
            name: name.to_owned(),
            family: family.to_owned(),
            script,
            faces,
            ligatures,
        }
    }

    fn empty(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            family: name.to_owned(),
            script: Script::Any,
            faces: Faces {
                regular: Face::system(),
                bold: Face::system(),
                italic: Face::system(),
                bold_italic: Face::system(),
            },
            ligatures: false,
        }
    }

    fn from_node(base: Self, node: Node<'_>) -> Self {
        Self {
            family: node
                .child("family")
                .and_then(|node| node.text())
                .map(str::to_owned)
                .unwrap_or(base.family),
            script: node
                .child("script")
                .and_then(|node| node.text())
                .and_then(Script::parse)
                .unwrap_or(base.script),
            ligatures: node
                .child("ligatures")
                .and_then(|node| node.boolean())
                .unwrap_or(base.ligatures),
            faces: Faces::from_node(base.faces, &base.name, node),
            ..base
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fonts {
    use_system: bool,
    named: Vec<NamedFont>,
    sets: FontSets,
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            use_system: false,
            named: vec![
                NamedFont::builtin(
                    "latin",
                    "Source Serif 4",
                    Script::Latin,
                    Faces {
                        regular: Face::embedded(EmbeddedFace::SourceSerifRegular),
                        bold: Face::embedded(EmbeddedFace::SourceSerifBold),
                        italic: Face::embedded(EmbeddedFace::SourceSerifItalic),
                        bold_italic: Face::system(),
                    },
                    false,
                ),
                NamedFont::builtin(
                    "cyrillic",
                    "Lora",
                    Script::Cyrillic,
                    Faces {
                        regular: Face::embedded(EmbeddedFace::LoraRegular),
                        bold: Face::embedded(EmbeddedFace::LoraBold),
                        italic: Face::embedded(EmbeddedFace::LoraItalic),
                        bold_italic: Face::system(),
                    },
                    false,
                ),
                NamedFont::builtin(
                    "code",
                    "JetBrainsMono Nerd Font",
                    Script::Any,
                    Faces {
                        regular: Face::embedded(EmbeddedFace::JetBrainsRegular),
                        bold: Face::system(),
                        italic: Face::system(),
                        bold_italic: Face::system(),
                    },
                    true,
                ),
            ],
            sets: FontSets::default(),
        }
    }
}

impl Fonts {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        let use_system = node
            .child("use-system")
            .and_then(|node| node.boolean())
            .unwrap_or(base.use_system);
        let sets = FontSets::from_node(base.sets, node.child("sets"));
        let mut fonts = Self {
            use_system,
            named: base.named,
            sets,
        };

        for child in node.children() {
            let Some(name) = child.name() else {
                continue;
            };

            if name == "use-system" || name == "sets" {
                continue;
            }

            let current = fonts
                .named(name)
                .cloned()
                .unwrap_or_else(|| NamedFont::empty(name));
            let decoded = NamedFont::from_node(current, child);

            match fonts.named.iter_mut().find(|font| font.name == name) {
                Some(font) => *font = decoded,
                None => fonts.named.push(decoded),
            }
        }

        fonts
    }

    pub fn use_system(&self) -> bool {
        self.use_system
    }

    pub fn named(&self, name: &str) -> Option<&NamedFont> {
        self.named.iter().find(|font| font.name == name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &NamedFont> {
        self.named.iter()
    }

    pub fn set(&self, name: &str) -> Option<&FontSet> {
        self.sets.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;
    use crate::config::face::FaceSource;
    use std::path::Path;

    #[test]
    fn script_classifies_latin_and_cyrillic() {
        assert_eq!(Script::of('A'), Script::Latin);
        assert_eq!(Script::of('я'), Script::Cyrillic);
        assert_eq!(Script::of('1'), Script::Common);
    }

    #[test]
    fn embedded_faces_are_available_by_default() {
        let fonts = Fonts::default();
        let latin = fonts.named("latin").expect("latin");
        let code = fonts.named("code").expect("code");

        assert_eq!(latin.family, "Source Serif 4");
        assert_eq!(latin.script, Script::Latin);
        assert!(latin.faces.regular.source.bytes().is_some());
        assert!(code.ligatures);
        assert_eq!(code.faces.bold.source, FaceSource::System);
    }

    #[test]
    fn config_overrides_font_fields() {
        let document = Document::parse(
            r##"
fonts {
    latin {
        family "Custom Serif"
        script "latin"
        regular "/tmp/custom.otf"
        bold "system"
    }
    code {
        ligatures #false
    }
}
"##,
        )
        .expect("document");
        let fonts = Fonts::from_node(Fonts::default(), document.root().child("fonts"));
        let latin = fonts.named("latin").expect("latin");
        let code = fonts.named("code").expect("code");

        assert_eq!(latin.family, "Custom Serif");
        assert_eq!(
            latin.faces.regular.source.path(),
            Some(Path::new("/tmp/custom.otf"))
        );
        assert_eq!(latin.faces.bold.source, FaceSource::System);
        assert!(!code.ligatures);
    }

    #[test]
    fn use_system_is_decoded() {
        let document = Document::parse("fonts { use-system #true }").expect("document");
        let fonts = Fonts::from_node(Fonts::default(), document.root().child("fonts"));

        assert!(fonts.use_system());
    }
}
