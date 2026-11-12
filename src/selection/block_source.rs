use std::ops::Range;
use std::sync::Arc;

use crate::markdown::{SourceSegment, Text};

use crate::markdown::{SourceIndex, SourceLine};

#[derive(Debug, Clone)]
pub struct BlockSource {
    block: usize,
    code: bool,
    heading: Option<usize>,
    index: Arc<SourceIndex>,
    copy_index: Arc<SourceIndex>,
    segments: Arc<[SourceSegment]>,
}

impl BlockSource {
    pub fn from_text(block: usize, index: Arc<SourceIndex>, text: &Text) -> Self {
        Self {
            block,
            code: false,
            heading: None,
            copy_index: Arc::clone(&index),
            index,
            segments: text.source_segments().into(),
        }
    }

    pub fn from_code(block: usize, index: Arc<SourceIndex>, lines: &[Text]) -> Self {
        let mut segments = Vec::with_capacity(lines.len());
        let mut logical = 0;

        for line in lines {
            logical += match segments.is_empty() {
                true => 0,
                false => 1,
            };
            let end = logical + line.source().len();
            segments.push(SourceSegment {
                logical: logical..end,
                source: line.source(),
            });
            logical = end;
        }

        Self {
            block,
            code: lines.len() > 1,
            heading: None,
            copy_index: Arc::clone(&index),
            index,
            segments: segments.into(),
        }
    }

    pub fn block(&self) -> usize {
        self.block
    }

    pub fn is_code(&self) -> bool {
        self.code
    }

    pub fn heading(self, level: usize) -> Self {
        Self {
            heading: Some(level),
            ..self
        }
    }

    pub fn heading_level(&self) -> Option<usize> {
        self.heading
    }

    pub fn index(&self) -> &Arc<SourceIndex> {
        &self.index
    }

    pub fn copy_index(&self) -> &Arc<SourceIndex> {
        &self.copy_index
    }

    pub fn with_copy_index(mut self, index: Arc<SourceIndex>) -> Self {
        self.copy_index = index;
        self
    }

    pub fn segments(&self) -> &[SourceSegment] {
        &self.segments
    }

    pub fn logical_len(&self) -> usize {
        self.segments
            .last()
            .map(|segment| segment.logical.end)
            .unwrap_or_default()
    }

    pub fn source_at(&self, logical: usize) -> usize {
        let Some(segment) = self
            .segments
            .iter()
            .rev()
            .find(|segment| logical >= segment.logical.start)
        else {
            return self
                .segments
                .first()
                .map(|segment| segment.source.start)
                .unwrap_or_default();
        };
        let offset = logical.saturating_sub(segment.logical.start);
        let source_len = segment.source.len();

        segment.source.start + offset.min(source_len)
    }

    pub fn line_at(&self, logical: usize) -> usize {
        self.index.line_of(self.source_at(logical))
    }

    pub fn line(&self, logical: usize) -> Option<SourceLine> {
        self.index.line(self.line_at(logical))
    }

    pub fn lines_in(&self, logical: Range<usize>) -> Range<usize> {
        let start = self.line_at(logical.start);
        let end = self.line_at(logical.end.saturating_sub(1));

        start..end.saturating_add(1)
    }

    pub fn source_range(&self, logical: Range<usize>) -> Range<usize> {
        let lines = self.lines_in(logical);

        self.index.range(lines.start, lines.end.saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::Article;
    use crate::markdown::Item;

    fn text(article: &Article) -> &Text {
        article
            .items()
            .iter()
            .find_map(|item| match item {
                Item::Paragraph(text) => Some(text),
                _ => None,
            })
            .expect("paragraph")
    }

    fn block(source: &str) -> (Article, BlockSource) {
        let article = Article::parse(source);
        let block = BlockSource::from_text(
            text(&article).source().start,
            article.index().clone(),
            text(&article),
        );

        (article, block)
    }

    #[test]
    fn plain_text_maps_to_source_bytes() {
        let (_article, block) = block("alpha beta");

        assert_eq!(block.source_at(0), 0);
        assert_eq!(block.source_at(6), 6);
        assert_eq!(block.line_at(0), 0);
        assert_eq!(block.heading_level(), None);
    }

    #[test]
    fn soft_break_maps_to_newline_source() {
        let (_article, block) = block("alpha\nbeta");

        assert_eq!(block.source_at(5), 5);
        assert_eq!(block.source_at(6), 6);
        assert_eq!(block.line_at(5), 0);
        assert_eq!(block.line_at(6), 1);
    }

    #[test]
    fn inline_code_maps_to_backticked_source() {
        let (_article, block) = block("alpha `code` omega");
        let logical = "alpha code omega".find("code").expect("code");
        let source = block.source_at(logical);

        assert!(source >= "alpha ".len());
        assert!(source <= "alpha `code".len());
    }

    fn collect_texts<'a>(items: &'a [Item], texts: &mut Vec<&'a Text>) {
        for item in items {
            match item {
                Item::Paragraph(text) | Item::Heading(_, text) => texts.push(text),
                Item::List { bullets, .. } => {
                    for bullet in bullets {
                        let items = match bullet {
                            crate::markdown::Bullet::Point { items }
                            | crate::markdown::Bullet::Task { items, .. } => items,
                        };
                        collect_texts(items, texts);
                    }
                }
                Item::Quote(items) => collect_texts(items, texts),
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
    fn nested_blocks_map_to_their_source_lines() {
        let source = "- point\n\n> quoted\n\n| head |\n| --- |\n| cell |\n";
        let article = Article::parse(source);
        let mut texts = Vec::new();
        collect_texts(article.items(), &mut texts);
        let index = article.index();
        let snippets = ["point", "quoted", "head", "cell"];

        for snippet in snippets {
            let text = texts
                .iter()
                .find(|text| {
                    text.source_segments().iter().any(|segment| {
                        index
                            .slice(segment.source.clone())
                            .is_some_and(|source| source.contains(snippet))
                    })
                })
                .unwrap_or_else(|| panic!("missing {snippet}"));
            let block = BlockSource::from_text(text.source().start, index.clone(), text);
            let line = block.line(0).expect("line");
            let content = index
                .slice(line.start()..line.content_end())
                .expect("line content");

            assert!(content.contains(snippet), "line={content}");
        }
    }

    #[test]
    fn heading_level_is_attached_to_block() {
        let (_article, block) = block("heading");

        assert_eq!(block.heading(2).heading_level(), Some(2));
    }

    #[test]
    fn code_block_source_excludes_fence_and_keeps_blank_lines() {
        let source = "```rust\nfn a() {}\n\nfn b() {}\n```\n";
        let article = Article::parse(source);
        let lines = article
            .items()
            .iter()
            .find_map(|item| match item {
                Item::CodeBlock { lines, .. } => Some(lines.clone()),
                _ => None,
            })
            .expect("code block");
        let block =
            BlockSource::from_code(lines[0].source().start, article.index().clone(), &lines);

        assert!(block.is_code());
        assert_eq!(lines.len(), 3);
        assert_eq!(
            article.index().slice(block.source_range(0..10)).unwrap(),
            "fn a() {}\n"
        );
        assert_eq!(block.line_at(block.logical_len()), 3);
        assert_eq!(
            article
                .index()
                .slice(block.source_range(block.logical_len() - 1..block.logical_len()))
                .unwrap(),
            "fn b() {}\n"
        );
    }

    #[test]
    fn single_line_fence_is_not_flagged_as_multiline_code() {
        let article = Article::parse("```\ncode\n```\n");
        let lines = article
            .items()
            .iter()
            .find_map(|item| match item {
                Item::CodeBlock { lines, .. } => Some(lines.clone()),
                _ => None,
            })
            .expect("code block");
        let block =
            BlockSource::from_code(lines[0].source().start, article.index().clone(), &lines);

        assert!(!block.is_code());
    }

    #[test]
    fn line_boundary_belongs_to_next_line() {
        let (_article, block) = block("alpha\nbeta");

        assert_eq!(block.lines_in(0..5), 0..1);
        assert_eq!(block.lines_in(0..6), 0..1);
        assert_eq!(block.lines_in(6..10), 1..2);
    }
}
