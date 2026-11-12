use std::path::PathBuf;

use crate::config::node::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconSource {
    Glyph(String),
    EmbeddedSvg(&'static [u8]),
    Svg(PathBuf),
}

impl IconSource {
    pub(crate) fn from_node(node: Node<'_>, glyph: &str, path: &str, base: Self) -> Self {
        if let Some(path) = node.child(path).and_then(|node| node.text()) {
            return Self::Svg(PathBuf::from(path));
        }

        if let Some(value) = node.child(glyph).and_then(|node| node.text()) {
            if value == "embedded" {
                return base;
            }

            return Self::Glyph(value.to_owned());
        }

        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn glyph_and_svg_sources_are_decoded() {
        let glyph_document = Document::parse("entry { icon \"A\" }").expect("document");
        let glyph_entry = glyph_document.root().child("entry").expect("entry");
        let path_document =
            Document::parse("entry { icon-path \"/tmp/a.svg\" }").expect("document");
        let path_entry = path_document.root().child("entry").expect("entry");

        assert_eq!(
            IconSource::from_node(
                glyph_entry,
                "icon",
                "icon-path",
                IconSource::Glyph("x".into())
            ),
            IconSource::Glyph("A".to_owned())
        );
        assert_eq!(
            IconSource::from_node(
                path_entry,
                "icon",
                "icon-path",
                IconSource::Glyph("x".into())
            ),
            IconSource::Svg(PathBuf::from("/tmp/a.svg"))
        );
    }
}
