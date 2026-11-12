use std::sync::Arc;

use crate::markdown::{Item, SourceIndex, parse};

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
            items: parse(source),
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

    fn paragraph(source: &str) -> crate::markdown::Text {
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
    fn crlf_soft_break_maps_to_both_line_ending_bytes() {
        let text = paragraph("alpha\r\nbeta");
        let segments = text.source_segments();

        assert_eq!(segments[1].logical, 5..6);
        assert_eq!(segments[1].source, 5..7);
        assert_eq!(segments[2].source, 7..11);
    }

    #[test]
    fn hard_break_maps_to_its_source_bytes() {
        let text = paragraph("alpha  \nbeta");
        let segments = text.source_segments();

        assert_eq!(segments[1].logical, 5..6);
        assert_eq!(segments[1].source, 5..8);
        assert_eq!(segments[2].source, 8..12);
    }

    #[test]
    fn empty_code_line_keeps_empty_source_range() {
        let source = "```\na\n\nb\n```\n";
        let article = Article::parse(source);
        let lines = article
            .items()
            .iter()
            .find_map(|item| match item {
                Item::CodeBlock { lines, .. } => Some(lines),
                _ => None,
            })
            .expect("code block");

        assert_eq!(lines.len(), 3);
        assert!(lines[1].source().is_empty());
        assert_eq!(&source[lines[0].source()], "a");
        assert_eq!(&source[lines[2].source()], "b");
    }

    #[test]
    fn several_text_events_split_one_block() {
        let text = paragraph("a *b* c");
        let texts: Vec<&str> = text
            .source_segments()
            .iter()
            .map(|segment| &"a b c"[segment.logical.clone()])
            .collect();

        assert_eq!(texts, vec!["a ", "b", " c"]);
    }

    #[test]
    fn multi_backtick_code_keeps_full_source_range() {
        let source = "a ``c`` b";
        let text = paragraph(source);
        let code = text
            .source_segments()
            .into_iter()
            .find(|segment| segment.logical == (2..3))
            .expect("code");

        assert_eq!(code.source, 2..7);
    }

    fn collect_texts<'a>(items: &'a [Item], texts: &mut Vec<&'a crate::markdown::Text>) {
        for item in items {
            match item {
                Item::Paragraph(text) | Item::Heading(_, text) => texts.push(text),
                Item::List { bullets, .. } => {
                    for bullet in bullets {
                        collect_texts(bullet.items(), texts);
                    }
                }
                Item::Quote(contents) => collect_texts(contents, texts),
                Item::Table { columns, rows } => {
                    for column in columns {
                        collect_texts(&column.header, texts);
                    }
                    for row in rows {
                        for cell in row.cells() {
                            collect_texts(cell, texts);
                        }
                    }
                }
                Item::CodeBlock { lines, .. } => texts.extend(lines),
                Item::Image { .. } | Item::Rule => {}
            }
        }
    }

    #[test]
    fn block_starts_are_unique_and_ordered() {
        let source = "| a | b |\n|---|---|\n| c | d |\n\n- point\n\n> quote\n";
        let article = Article::parse(source);
        let mut texts = Vec::new();
        collect_texts(article.items(), &mut texts);
        let starts: Vec<usize> = texts.iter().map(|text| text.source().start).collect();

        for pair in starts.windows(2) {
            assert!(pair[0] < pair[1], "starts={starts:?}");
        }

        for pair in texts.windows(2) {
            assert!(pair[0].source().end <= pair[1].source().start);
        }
    }

    #[test]
    fn source_index_matches_parsed_source() {
        let article = Article::parse(FIXTURE);

        assert_eq!(article.index().source(), FIXTURE);
        assert_eq!(article.index().line_count(), FIXTURE.lines().count());
        assert!(article.index().range(0, 0).end <= FIXTURE.len());
    }
}
