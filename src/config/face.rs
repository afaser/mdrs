use std::path::{Path, PathBuf};

use crate::config::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedFace {
    LoraRegular,
    LoraBold,
    LoraItalic,
    LoraBoldItalic,
    SourceSerifRegular,
    SourceSerifBold,
    SourceSerifItalic,
    SourceSerifBoldItalic,
    JetBrainsRegular,
    JetBrainsBold,
    JetBrainsItalic,
    JetBrainsBoldItalic,
}

impl EmbeddedFace {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::LoraRegular => include_bytes!("../../assets/Lora-Regular.otf"),
            Self::LoraBold => include_bytes!("../../assets/Lora-Bold.otf"),
            Self::LoraItalic => include_bytes!("../../assets/Lora-Italic.otf"),
            Self::LoraBoldItalic => include_bytes!("../../assets/Lora-BoldItalic.otf"),
            Self::SourceSerifRegular => include_bytes!("../../assets/SourceSerif4-Regular.otf"),
            Self::SourceSerifBold => include_bytes!("../../assets/SourceSerif4-Bold.otf"),
            Self::SourceSerifItalic => include_bytes!("../../assets/SourceSerif4-It.otf"),
            Self::SourceSerifBoldItalic => include_bytes!("../../assets/SourceSerif4-BoldIt.otf"),
            Self::JetBrainsRegular => {
                include_bytes!("../../assets/JetBrainsMonoNerdFont-Regular.ttf")
            }
            Self::JetBrainsBold => include_bytes!("../../assets/JetBrainsMonoNerdFont-Bold.ttf"),
            Self::JetBrainsItalic => {
                include_bytes!("../../assets/JetBrainsMonoNerdFont-Italic.ttf")
            }
            Self::JetBrainsBoldItalic => {
                include_bytes!("../../assets/JetBrainsMonoNerdFont-BoldItalic.ttf")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaceSource {
    Embedded(EmbeddedFace),
    System,
    Path(PathBuf),
}

impl FaceSource {
    pub fn bytes(&self) -> Option<&'static [u8]> {
        match self {
            Self::Embedded(face) => Some(face.bytes()),
            Self::System | Self::Path(_) => None,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(path),
            Self::Embedded(_) | Self::System => None,
        }
    }

    fn parse(value: &str, font: &str, kind: FaceKind) -> Option<Self> {
        match value {
            "embedded" => embedded_face(font, kind),
            "system" => Some(Self::System),
            path => Some(Self::Path(PathBuf::from(path))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FaceKind {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Face {
    pub source: FaceSource,
}

impl Face {
    pub fn embedded(face: EmbeddedFace) -> Self {
        Self {
            source: FaceSource::Embedded(face),
        }
    }

    pub fn system() -> Self {
        Self {
            source: FaceSource::System,
        }
    }

    fn from_node(node: Node<'_>, name: &str, font: &str, kind: FaceKind, base: Self) -> Self {
        let Some(value) = node.child(name).and_then(|node| node.text()) else {
            return base;
        };

        FaceSource::parse(value, font, kind)
            .map(|source| Self { source })
            .unwrap_or(base)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Faces {
    pub regular: Face,
    pub bold: Face,
    pub italic: Face,
    pub bold_italic: Face,
}

impl Faces {
    pub fn from_node(base: Self, font: &str, node: Node<'_>) -> Self {
        Self {
            regular: Face::from_node(node, "regular", font, FaceKind::Regular, base.regular),
            bold: Face::from_node(node, "bold", font, FaceKind::Bold, base.bold),
            italic: Face::from_node(node, "italic", font, FaceKind::Italic, base.italic),
            bold_italic: Face::from_node(
                node,
                "bold-italic",
                font,
                FaceKind::BoldItalic,
                base.bold_italic,
            ),
        }
    }
}

fn embedded_face(font: &str, kind: FaceKind) -> Option<FaceSource> {
    let face = match (font, kind) {
        ("latin", FaceKind::Regular) => EmbeddedFace::SourceSerifRegular,
        ("latin", FaceKind::Bold) => EmbeddedFace::SourceSerifBold,
        ("latin", FaceKind::Italic) => EmbeddedFace::SourceSerifItalic,
        ("latin", FaceKind::BoldItalic) => EmbeddedFace::SourceSerifBoldItalic,
        ("cyrillic", FaceKind::Regular) => EmbeddedFace::LoraRegular,
        ("cyrillic", FaceKind::Bold) => EmbeddedFace::LoraBold,
        ("cyrillic", FaceKind::Italic) => EmbeddedFace::LoraItalic,
        ("cyrillic", FaceKind::BoldItalic) => EmbeddedFace::LoraBoldItalic,
        ("code", FaceKind::Regular) => EmbeddedFace::JetBrainsRegular,
        ("code", FaceKind::Bold) => EmbeddedFace::JetBrainsBold,
        ("code", FaceKind::Italic) => EmbeddedFace::JetBrainsItalic,
        ("code", FaceKind::BoldItalic) => EmbeddedFace::JetBrainsBoldItalic,
        _ => return None,
    };

    Some(FaceSource::Embedded(face))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn embedded_sources_expose_bytes() {
        assert!(
            FaceSource::Embedded(EmbeddedFace::LoraRegular)
                .bytes()
                .is_some()
        );
        assert!(FaceSource::System.bytes().is_none());
    }

    #[test]
    fn face_sources_are_decoded_per_kind() {
        let document = Document::parse(
            r#"
latin {
    regular "/tmp/custom.otf"
    bold "system"
    italic "embedded"
}
"#,
        )
        .expect("document");
        let node = document.root().child("latin").expect("latin");
        let faces = Faces::from_node(
            Faces {
                regular: Face::system(),
                bold: Face::system(),
                italic: Face::system(),
                bold_italic: Face::system(),
            },
            "latin",
            node,
        );

        assert_eq!(
            faces.regular.source.path(),
            Some(Path::new("/tmp/custom.otf"))
        );
        assert_eq!(faces.bold.source, FaceSource::System);
        assert_eq!(
            faces.italic.source,
            FaceSource::Embedded(EmbeddedFace::SourceSerifItalic)
        );
    }
}
