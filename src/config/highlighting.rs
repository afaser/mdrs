use std::path::PathBuf;

use crate::config::colors::ColorRef;
use crate::config::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenClass {
    Keyword,
    String,
    Comment,
    Number,
    Function,
    Type,
    Operator,
    Punctuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Tokens {
    pub keyword: Option<ColorRef>,
    pub string: Option<ColorRef>,
    pub comment: Option<ColorRef>,
    pub number: Option<ColorRef>,
    pub function: Option<ColorRef>,
    pub kind: Option<ColorRef>,
    pub operator: Option<ColorRef>,
    pub punctuation: Option<ColorRef>,
}

impl Tokens {
    pub fn color(&self, class: TokenClass) -> Option<ColorRef> {
        match class {
            TokenClass::Keyword => self.keyword,
            TokenClass::String => self.string,
            TokenClass::Comment => self.comment,
            TokenClass::Number => self.number,
            TokenClass::Function => self.function,
            TokenClass::Type => self.kind,
            TokenClass::Operator => self.operator,
            TokenClass::Punctuation => self.punctuation,
        }
    }

    fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            keyword: color(node, "keyword", base.keyword),
            string: color(node, "string", base.string),
            comment: color(node, "comment", base.comment),
            number: color(node, "number", base.number),
            function: color(node, "function", base.function),
            kind: color(node, "type", base.kind),
            operator: color(node, "operator", base.operator),
            punctuation: color(node, "punctuation", base.punctuation),
        }
    }
}

impl Tokens {
    pub fn is_empty(&self) -> bool {
        self.keyword.is_none()
            && self.string.is_none()
            && self.comment.is_none()
            && self.number.is_none()
            && self.function.is_none()
            && self.kind.is_none()
            && self.operator.is_none()
            && self.punctuation.is_none()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Highlighting {
    pub theme: String,
    pub theme_file: Option<PathBuf>,
    pub tokens: Tokens,
}

impl Default for Highlighting {
    fn default() -> Self {
        Self {
            theme: "CatppuccinMocha".to_owned(),
            theme_file: None,
            tokens: Tokens::default(),
        }
    }
}

impl Highlighting {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            theme: node
                .child("theme")
                .and_then(|node| node.text())
                .map(str::to_owned)
                .unwrap_or(base.theme),
            theme_file: node
                .child("theme-file")
                .and_then(|node| node.text())
                .map(PathBuf::from),
            tokens: Tokens::from_node(base.tokens, node.child("tokens")),
        }
    }
}

fn color(node: Node<'_>, name: &str, base: Option<ColorRef>) -> Option<ColorRef> {
    node.child(name).and_then(|node| node.color()).or(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn token_overrides_are_decoded() {
        let document = Document::parse(
            r##"
highlighting {
    tokens {
        keyword "#ff0000"
        string "#00ff00"
    }
}
"##,
        )
        .expect("document");
        let highlighting = Highlighting::from_node(
            Highlighting::default(),
            document.root().child("highlighting"),
        );

        assert_eq!(highlighting.tokens.keyword, ColorRef::parse("#ff0000"));
        assert_eq!(highlighting.tokens.string, ColorRef::parse("#00ff00"));
        assert_eq!(highlighting.tokens.comment, None);
    }

    #[test]
    fn theme_file_takes_priority_in_config() {
        let document = Document::parse(
            r#"
highlighting {
    theme "Nord"
    theme-file "/tmp/custom.tmTheme"
}
"#,
        )
        .expect("document");
        let highlighting = Highlighting::from_node(
            Highlighting::default(),
            document.root().child("highlighting"),
        );

        assert_eq!(highlighting.theme, "Nord");
        assert_eq!(
            highlighting.theme_file,
            Some(PathBuf::from("/tmp/custom.tmTheme"))
        );
    }
}
