use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

use crate::config::colors::ColorRef;

#[derive(Debug, Clone, Copy)]
pub struct Node<'a> {
    content: Content<'a>,
}

#[derive(Debug, Clone, Copy)]
enum Content<'a> {
    Document(&'a KdlDocument),
    Node(&'a KdlNode),
}

impl<'a> Node<'a> {
    pub fn root(document: &'a KdlDocument) -> Self {
        Self {
            content: Content::Document(document),
        }
    }

    fn from_node(node: &'a KdlNode) -> Self {
        Self {
            content: Content::Node(node),
        }
    }

    pub fn child(&self, name: &str) -> Option<Self> {
        self.document()?
            .nodes()
            .iter()
            .rev()
            .find(|node| node.name().value() == name)
            .map(Self::from_node)
    }

    pub fn children(&self) -> Vec<Self> {
        self.document()
            .map(|document| document.nodes().iter().map(Self::from_node).collect())
            .unwrap_or_default()
    }

    pub fn name(&self) -> Option<&'a str> {
        match self.content {
            Content::Document(_) => None,
            Content::Node(node) => Some(node.name().value()),
        }
    }

    pub fn text(&self) -> Option<&'a str> {
        self.entries()
            .iter()
            .find(|entry| entry.name().is_none())
            .and_then(|entry| entry.value().as_string())
    }

    pub fn number(&self) -> Option<f32> {
        let value = self
            .entries()
            .iter()
            .find(|entry| entry.name().is_none())?
            .value();

        match value {
            KdlValue::Integer(value) => Some(*value as f32),
            KdlValue::Float(value) => Some(*value as f32),
            KdlValue::String(value) => value.parse().ok(),
            KdlValue::Bool(_) | KdlValue::Null => None,
        }
    }

    pub fn boolean(&self) -> Option<bool> {
        self.entries()
            .iter()
            .find(|entry| entry.name().is_none())
            .and_then(|entry| entry.value().as_bool())
    }

    pub fn color(&self) -> Option<ColorRef> {
        self.text().and_then(ColorRef::parse)
    }

    pub fn number_child(&self, name: &str) -> Option<f32> {
        self.child(name).and_then(|node| node.number())
    }

    pub fn boolean_child(&self, name: &str) -> Option<bool> {
        self.child(name).and_then(|node| node.boolean())
    }

    pub fn text_child(&self, name: &str) -> Option<&'a str> {
        self.child(name).and_then(|node| node.text())
    }

    pub fn color_child(&self, name: &str) -> Option<ColorRef> {
        self.child(name).and_then(|node| node.color())
    }

    pub fn arguments(&self) -> Vec<&'a str> {
        self.entries()
            .iter()
            .filter(|entry| entry.name().is_none())
            .filter_map(|entry| entry.value().as_string())
            .collect()
    }

    pub fn numbers(&self) -> Vec<f32> {
        self.entries()
            .iter()
            .filter(|entry| entry.name().is_none())
            .filter_map(|entry| match entry.value() {
                KdlValue::Integer(value) => Some(*value as f32),
                KdlValue::Float(value) => Some(*value as f32),
                KdlValue::String(value) => value.parse().ok(),
                KdlValue::Bool(_) | KdlValue::Null => None,
            })
            .collect()
    }

    fn document(&self) -> Option<&'a KdlDocument> {
        match self.content {
            Content::Document(document) => Some(document),
            Content::Node(node) => node.children(),
        }
    }

    fn entries(&self) -> &'a [KdlEntry] {
        match self.content {
            Content::Document(_) => &[],
            Content::Node(node) => node.entries(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::document::Document;

    #[test]
    fn last_repeated_field_wins() {
        let document =
            Document::parse("theme { palette { accent \"#111111\"; accent \"#222222\" } }")
                .expect("document");
        let root = document.root();
        let accent = root
            .child("theme")
            .and_then(|theme| theme.child("palette"))
            .and_then(|palette| palette.child("accent"))
            .and_then(|accent| accent.text());

        assert_eq!(accent, Some("#222222"));
    }

    #[test]
    fn nested_children_are_reachable() {
        let document =
            Document::parse("theme { code-block { header { copy { color \"#ff0000\" } } } }")
                .expect("document");
        let root = document.root();
        let color = root
            .child("theme")
            .and_then(|theme| theme.child("code-block"))
            .and_then(|code| code.child("header"))
            .and_then(|header| header.child("copy"))
            .and_then(|copy| copy.child("color"))
            .and_then(|color| color.text());

        assert_eq!(color, Some("#ff0000"));
    }

    #[test]
    fn unknown_nodes_are_ignored() {
        let document = Document::parse(
            "theme { future-branch { enabled #true }; palette { accent \"#ff0000\" } }",
        )
        .expect("document");
        let root = document.root();
        let palette = root
            .child("theme")
            .and_then(|theme| theme.child("palette"))
            .expect("palette");

        assert_eq!(
            palette.child("accent").and_then(|accent| accent.text()),
            Some("#ff0000")
        );
        assert!(
            root.child("theme")
                .and_then(|theme| theme.child("future-branch"))
                .is_some()
        );
    }

    #[test]
    fn number_accepts_integer_and_float() {
        let document = Document::parse("a 4\nb 4.5").expect("document");
        let root = document.root();

        assert_eq!(root.child("a").and_then(|node| node.number()), Some(4.0));
        assert_eq!(root.child("b").and_then(|node| node.number()), Some(4.5));
    }
}
