use std::ops::Range;

use iced::Point;

use crate::selection::document_geometry::{DocumentGeometry, GeometryHit};
use crate::selection::generation::Generation;

const DRAG_THRESHOLD: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextPosition {
    pub block: usize,
    pub byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LinePosition {
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Text,
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Position {
    Text(TextPosition),
    Line(LinePosition),
}

#[derive(Debug, Clone, Copy)]
struct Press {
    origin: Point,
    moved: bool,
}

#[derive(Debug, Clone)]
pub struct DocumentSelection {
    anchor: Option<Position>,
    focus: Option<Position>,
    mode: SelectionMode,
    press: Option<Press>,
    generation: Generation,
}

impl Default for DocumentSelection {
    fn default() -> Self {
        Self {
            anchor: None,
            focus: None,
            mode: SelectionMode::Text,
            press: None,
            generation: Generation::default(),
        }
    }
}

impl DocumentSelection {
    pub fn sync_generation(&mut self, generation: Generation) {
        if generation == self.generation {
            return;
        }

        self.generation = generation;
        self.clear();
    }

    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    pub fn begin_text(&mut self, block: usize, byte: usize, origin: Point) {
        self.anchor = Some(Position::Text(TextPosition { block, byte }));
        self.focus = self.anchor;
        self.mode = SelectionMode::Text;
        self.press = Some(Press {
            origin,
            moved: false,
        });
    }

    pub fn begin_line(&mut self, line: usize, origin: Point) {
        self.anchor = Some(Position::Line(LinePosition { line }));
        self.focus = self.anchor;
        self.mode = SelectionMode::Line;
        self.press = Some(Press {
            origin,
            moved: false,
        });
    }

    pub fn extend(&mut self, hit: GeometryHit, point: Point) {
        let Some(press) = self.press.as_mut() else {
            return;
        };

        let position = match self.mode {
            SelectionMode::Text => Position::Text(TextPosition {
                block: hit.block,
                byte: hit.byte,
            }),
            SelectionMode::Line => Position::Line(LinePosition { line: hit.line }),
        };

        self.focus = Some(position);

        if point.distance(press.origin) > DRAG_THRESHOLD {
            press.moved = true;
        }
    }

    pub fn finish(&mut self, geometry: &DocumentGeometry) -> Option<String> {
        let press = self.press.take()?;
        let line_click = self.mode == SelectionMode::Line && self.ordered_lines().is_some();

        if !press.moved && !line_click {
            self.clear();
            return None;
        }

        let index = geometry.copy_index()?;
        let range = match self.mode {
            SelectionMode::Line => {
                let (start, end) = self.ordered_lines()?;
                index.range(start.line, end.line)
            }
            SelectionMode::Text => {
                let (start, end) = self.ordered_text()?;
                let start_source = geometry.source(start.block)?;
                let end_source = geometry.source(end.block)?;
                let start_line = start_source.line_at(start.byte);
                let end_line = end_source.line_at(end.byte);

                index.range(start_line, end_line)
            }
        };
        index
            .slice(range)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    }

    pub fn clear(&mut self) {
        self.clear_positions();
        self.press = None;
    }

    pub fn dragging(&self) -> bool {
        self.press.is_some()
    }

    pub fn dragged(&self) -> bool {
        self.press.is_some_and(|press| press.moved)
    }

    pub fn text_range(&self, block: usize, len: usize) -> Option<Range<usize>> {
        if self.mode != SelectionMode::Text {
            return None;
        }

        let (start, end) = self.ordered_text()?;

        Some(match (block.cmp(&start.block), block.cmp(&end.block)) {
            (std::cmp::Ordering::Less, _) | (_, std::cmp::Ordering::Greater) => return None,
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => start.byte..end.byte,
            (std::cmp::Ordering::Equal, _) => start.byte..len,
            (_, std::cmp::Ordering::Equal) => 0..end.byte,
            _ => 0..len,
        })
    }

    pub fn line_range(&self) -> Option<Range<usize>> {
        if self.mode != SelectionMode::Line {
            return None;
        }

        let (start, end) = self.ordered_lines()?;

        Some(start.line..end.line.saturating_add(1))
    }

    fn ordered_text(&self) -> Option<(TextPosition, TextPosition)> {
        let Position::Text(anchor) = self.anchor? else {
            return None;
        };
        let Position::Text(focus) = self.focus? else {
            return None;
        };

        Some(match anchor <= focus {
            true => (anchor, focus),
            false => (focus, anchor),
        })
    }

    fn ordered_lines(&self) -> Option<(LinePosition, LinePosition)> {
        let Position::Line(anchor) = self.anchor? else {
            return None;
        };
        let Position::Line(focus) = self.focus? else {
            return None;
        };

        Some(match anchor.line <= focus.line {
            true => (anchor, focus),
            false => (focus, anchor),
        })
    }

    fn clear_positions(&mut self) {
        self.anchor = None;
        self.focus = None;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::markdown::{Article, SourceIndex};
    use crate::selection::block_source::BlockSource;
    use crate::selection::document_geometry::VisualLine;

    fn article(source: &str) -> (Article, Vec<BlockSource>) {
        let article = Article::parse(source);
        let blocks = article
            .items()
            .iter()
            .filter_map(|item| match item {
                iced::widget::markdown::Item::Paragraph(text) => Some(BlockSource::from_text(
                    text.source().start,
                    article.index().clone(),
                    text,
                )),
                _ => None,
            })
            .collect();

        (article, blocks)
    }

    fn geometry(article: &Article, blocks: &[BlockSource]) -> DocumentGeometry {
        let mut geometry = DocumentGeometry::default();
        let index = article.index().clone();

        for block in blocks {
            let line = index.line(block.line_at(0)).expect("line");
            geometry.register(
                Generation::default(),
                block.clone(),
                vec![VisualLine {
                    top: 0.0,
                    height: 20.0,
                    left: 0.0,
                    logical: 0..block.logical_len(),
                    source: line.start()..line.end(),
                    glyphs: Vec::new(),
                }],
            );
        }

        geometry
    }

    fn text_position(block: usize, byte: usize) -> TextPosition {
        TextPosition { block, byte }
    }

    #[test]
    fn forward_and_reverse_text_ranges_match() {
        let (article, blocks) = article("alpha\n\nbeta");
        let first = blocks[0].block();
        let second = blocks[1].block();
        let mut forward = DocumentSelection::default();
        forward.begin_text(first, 1, Point::ORIGIN);
        forward.extend(
            GeometryHit {
                block: second,
                line: 0,
                byte: 2,
                gutter: false,
            },
            Point::new(10.0, 10.0),
        );
        let mut reverse = DocumentSelection::default();
        reverse.begin_text(second, 2, Point::ORIGIN);
        reverse.extend(
            GeometryHit {
                block: first,
                line: 0,
                byte: 1,
                gutter: false,
            },
            Point::new(10.0, 10.0),
        );

        assert_eq!(forward.text_range(first, 5), Some(1..5));
        assert_eq!(reverse.text_range(first, 5), Some(1..5));
        assert_eq!(forward.text_range(second, 4), Some(0..2));
        assert_eq!(reverse.text_range(second, 4), Some(0..2));
        let _ = (article, text_position(first, 1));
    }

    #[test]
    fn text_selection_copies_full_source_lines() {
        let (article, blocks) = article("alpha\n\nbeta");
        let geometry = geometry(&article, &blocks);
        let mut selection = DocumentSelection::default();
        selection.begin_text(blocks[0].block(), 1, Point::ORIGIN);
        selection.extend(
            GeometryHit {
                block: blocks[1].block(),
                line: 0,
                byte: 2,
                gutter: false,
            },
            Point::new(10.0, 10.0),
        );

        assert_eq!(
            selection.finish(&geometry).as_deref(),
            Some("alpha\n\nbeta")
        );
    }

    #[test]
    fn line_selection_copies_exact_line_range() {
        let (article, blocks) = article("alpha\nbeta\ngamma");
        let geometry = geometry(&article, &blocks);
        let mut selection = DocumentSelection::default();
        selection.begin_line(0, Point::ORIGIN);
        selection.extend(
            GeometryHit {
                block: blocks[0].block(),
                line: 2,
                byte: 0,
                gutter: true,
            },
            Point::new(10.0, 10.0),
        );

        assert_eq!(
            selection.finish(&geometry).as_deref(),
            Some("alpha\nbeta\ngamma")
        );
    }

    #[test]
    fn click_without_drag_clears_selection() {
        let (article, blocks) = article("alpha");
        let geometry = geometry(&article, &blocks);
        let mut selection = DocumentSelection::default();
        selection.begin_text(blocks[0].block(), 1, Point::ORIGIN);

        assert_eq!(selection.finish(&geometry), None);
        assert!(selection.text_range(blocks[0].block(), 5).is_none());
    }

    #[test]
    fn line_click_selects_exactly_one_line_and_keeps_range() {
        let (article, blocks) = article(
            "alpha
beta
gamma",
        );
        let geometry = geometry(&article, &blocks);
        let mut selection = DocumentSelection::default();
        selection.begin_line(1, Point::ORIGIN);

        assert_eq!(selection.finish(&geometry).as_deref(), Some("beta\n"));
        assert_eq!(selection.line_range(), Some(1..2));
    }

    #[test]
    fn text_range_outside_selected_blocks_is_empty() {
        let (_article, blocks) = article("alpha\n\nbeta");
        let mut selection = DocumentSelection::default();
        selection.begin_text(blocks[0].block(), 1, Point::ORIGIN);
        selection.extend(
            GeometryHit {
                block: blocks[1].block(),
                line: 0,
                byte: 2,
                gutter: false,
            },
            Point::new(10.0, 10.0),
        );

        assert_eq!(selection.text_range(999, 5), None);
        assert_eq!(selection.text_range(blocks[0].block(), 5), Some(1..5));
    }

    #[test]
    fn geometry_width_does_not_change_logical_range() {
        let (article, blocks) = article("alpha\n\nbeta");
        let mut geometry = geometry(&article, &blocks);
        let mut selection = DocumentSelection::default();
        selection.begin_text(blocks[0].block(), 1, Point::ORIGIN);
        selection.extend(
            GeometryHit {
                block: blocks[1].block(),
                line: 0,
                byte: 2,
                gutter: false,
            },
            Point::new(10.0, 10.0),
        );
        let before = selection.text_range(blocks[0].block(), 5);
        geometry.set_content_width(100.0);

        assert_eq!(selection.text_range(blocks[0].block(), 5), before);
        assert_eq!(selection.text_range(blocks[1].block(), 4), Some(0..2));
    }

    #[test]
    fn generation_change_clears_range() {
        let (article, blocks) = article("alpha");
        let mut selection = DocumentSelection::default();
        selection.begin_text(blocks[0].block(), 1, Point::ORIGIN);
        let mut generation = Generation::default();
        generation.advance();
        selection.sync_generation(generation);

        assert!(!selection.dragging());
        assert!(
            selection
                .text_range(blocks[0].block(), article.index().len())
                .is_none()
        );
    }

    #[test]
    fn source_index_is_shared() {
        let index = Arc::new(SourceIndex::new("alpha"));

        assert_eq!(index.slice(0..5), Some("alpha"));
    }
}
