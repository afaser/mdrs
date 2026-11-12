use std::sync::Arc;

use iced::widget::markdown::{self, Item};

use crate::markdown::SourceIndex;

#[derive(Debug, Clone)]
pub struct Article {
    index: Arc<SourceIndex>,
    copy_index: Arc<SourceIndex>,
    items: Vec<Item>,
}

impl Article {
    #[cfg(test)]
    pub fn parse(source: &str) -> Self {
        Self::parse_with_source(source, source)
    }

    pub fn parse_with_source(copy_source: &str, source: &str) -> Self {
        Self {
            index: Arc::new(SourceIndex::new(source)),
            copy_index: Arc::new(SourceIndex::new(copy_source)),
            items: markdown::parse(source).collect(),
        }
    }

    pub fn index(&self) -> &Arc<SourceIndex> {
        &self.index
    }

    pub fn copy_index(&self) -> &Arc<SourceIndex> {
        &self.copy_index
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
        assert!(article.index().is_empty());
    }

    fn paragraph(source: &str) -> iced::widget::markdown::Text {
        Article::parse(source)
            .items()
            .iter()
            .find_map(|item| match item {
                Item::Paragraph(text) => Some(text.clone()),
                _ => None,
            })
            .expect("paragraph")
    }

    #[test]
    fn plain_span_maps_logical_text_to_source() {
        let text = paragraph("alpha beta");
        let segments = text.source_segments();

        assert_eq!(text.source(), 0..10);
        assert_eq!(segments[0].logical, 0..10);
        assert_eq!(segments[0].source, 0..10);
    }

    #[test]
    fn soft_break_maps_inserted_space_to_newline_source() {
        let text = paragraph("alpha\nbeta");
        let segments = text.source_segments();

        assert_eq!(segments[0].source, 0..5);
        assert_eq!(segments[1].logical, 5..6);
        assert_eq!(segments[1].source, 5..6);
        assert_eq!(segments[2].source, 6..10);
    }

    #[test]
    fn inline_code_keeps_source_range_with_backticks() {
        let text = paragraph("alpha `code` omega");
        let code = text
            .source_segments()
            .into_iter()
            .find(|segment| segment.logical == (6..10))
            .expect("code segment");

        assert_eq!(code.source, 6..12);
    }

    #[test]
    fn code_lines_exclude_fence_and_header() {
        let source = "```rust\nfn main() {}\n```\n";
        let article = Article::parse(source);
        let lines = article
            .items()
            .iter()
            .find_map(|item| match item {
                Item::CodeBlock { lines, .. } => Some(lines),
                _ => None,
            })
            .expect("code block");

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].source(), 8..20);
        assert_eq!(&source[lines[0].source()], "fn main() {}");
    }

    #[test]
    fn copy_index_keeps_original_while_parse_uses_resolved_source() {
        let original = "![plot](images/plot.png)";
        let resolved = "![plot](file:///docs/images/plot.png)";
        let article = Article::parse_with_source(original, resolved);

        assert_eq!(article.index().source(), resolved);
        assert_eq!(article.copy_index().source(), original);
        assert_eq!(
            article.copy_index().slice(0..original.len()),
            Some(original)
        );
    }

    #[test]
    fn source_index_matches_parsed_source() {
        let article = Article::parse(FIXTURE);

        assert_eq!(article.index().source(), FIXTURE);
        assert_eq!(article.index().line_count(), FIXTURE.lines().count());
        assert!(article.index().range(0, 0).end <= FIXTURE.len());
    }
}
