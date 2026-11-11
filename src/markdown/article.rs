use iced::widget::markdown::{self, Item};

#[derive(Debug, Clone)]
pub struct Article {
    items: Vec<Item>,
}

impl Article {
    pub fn parse(source: &str) -> Self {
        Self {
            items: markdown::parse(source).collect(),
        }
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "# Heading\n\nParagraph with [link](https://example.com).\n\n![image](pic.png)\n\n> quote\n\n- point\n- point\n\n1. one\n2. two\n\n---\n\n```rust\nfn main() {}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";

    #[test]
    fn fixture_contains_every_item_variant() {
        let article = Article::parse(FIXTURE);

        assert!(
            article
                .items()
                .iter()
                .any(|item| matches!(item, Item::Heading(..)))
        );
        assert!(
            article
                .items()
                .iter()
                .any(|item| matches!(item, Item::Paragraph(_)))
        );
        assert!(
            article
                .items()
                .iter()
                .any(|item| matches!(item, Item::CodeBlock { .. }))
        );
        assert!(
            article
                .items()
                .iter()
                .any(|item| matches!(item, Item::List { .. }))
        );
        assert!(
            article
                .items()
                .iter()
                .any(|item| matches!(item, Item::Image { .. }))
        );
        assert!(
            article
                .items()
                .iter()
                .any(|item| matches!(item, Item::Quote(_)))
        );
        assert!(
            article
                .items()
                .iter()
                .any(|item| matches!(item, Item::Rule))
        );
        assert!(
            article
                .items()
                .iter()
                .any(|item| matches!(item, Item::Table { .. }))
        );
    }

    #[test]
    fn empty_source_produces_no_items() {
        let article = Article::parse("");

        assert!(article.items().is_empty());
    }
}
