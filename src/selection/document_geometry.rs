use std::collections::HashMap;
use std::ops::Range;

use iced::{Point, Rectangle};

use crate::markdown::SourceIndex;
use crate::selection::block_source::BlockSource;
use crate::selection::generation::Generation;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    pub x: f32,
    pub width: f32,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct VisualLine {
    pub top: f32,
    pub height: f32,
    pub left: f32,
    pub logical: Range<usize>,
    pub source: Range<usize>,
    pub glyphs: Vec<Glyph>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometryHit {
    pub block: usize,
    pub line: usize,
    pub byte: usize,
    pub gutter: bool,
}

#[derive(Debug, Clone)]
pub struct BlockGeometry {
    source: BlockSource,
    origin: Point,
    gutter: f32,
    lines: Vec<VisualLine>,
}

impl BlockGeometry {
    pub fn new(source: BlockSource, lines: Vec<VisualLine>) -> Self {
        let gutter = lines.iter().map(|line| line.left).fold(0.0, f32::max);

        Self {
            source,
            origin: Point::ORIGIN,
            gutter,
            lines,
        }
    }

    fn gutter_right(&self) -> f32 {
        self.origin.x + self.gutter
    }

    pub fn source(&self) -> &BlockSource {
        &self.source
    }

    pub fn is_code(&self) -> bool {
        self.source.is_code()
    }

    pub fn origin(&self) -> Point {
        self.origin
    }

    pub fn lines(&self) -> &[VisualLine] {
        &self.lines
    }

    pub fn line_rects(
        &self,
        lines: Range<usize>,
        height: f32,
        content_width: f32,
    ) -> Vec<Rectangle> {
        let width =
            content_width.max(self.origin.x + self.lines.first().map_or(0.0, |line| line.left));
        let index = self.source.index();
        let source = index.range(lines.start, lines.end.saturating_sub(1));

        self.lines
            .iter()
            .filter(|line| line.source.start < source.end && line.source.end > source.start)
            .map(|line| self.rect_for_line(line, height, width))
            .collect()
    }

    fn rect_for_line(&self, line: &VisualLine, height: f32, width: f32) -> Rectangle {
        let top = line.top + (line.height - height) / 2.0;

        Rectangle::new(
            Point::new(-self.origin.x, top),
            iced::Size::new(width, height),
        )
    }

    pub fn text_rects(&self, logical: Range<usize>, height: f32) -> Vec<Rectangle> {
        self.lines
            .iter()
            .filter_map(|line| {
                let start = logical.start.max(line.logical.start);
                let end = logical.end.min(line.logical.end);

                if start >= end {
                    return None;
                }

                let x = line_x(line, start);
                let width = line_x(line, end) - x;
                let top = line.top + (line.height - height) / 2.0;

                Some(Rectangle::new(
                    Point::new(x, top),
                    iced::Size::new(width, height),
                ))
            })
            .collect()
    }
}

fn line_x(line: &VisualLine, byte: usize) -> f32 {
    let Some(first) = line.glyphs.first() else {
        return line.left;
    };

    if byte <= first.start {
        return first.x;
    }

    let Some(last) = line.glyphs.last() else {
        return line.left;
    };

    if byte >= last.end {
        return last.x + last.width;
    }

    for glyph in &line.glyphs {
        if byte < glyph.start {
            return glyph.x;
        }

        if byte > glyph.end {
            continue;
        }

        let length = glyph.end.saturating_sub(glyph.start).max(1);
        let fraction = (byte - glyph.start) as f32 / length as f32;

        return glyph.x + glyph.width * fraction;
    }

    line.left
}

#[derive(Debug, Default)]
pub struct DocumentGeometry {
    generation: Generation,
    blocks: Vec<BlockGeometry>,
    line_owners: HashMap<usize, usize>,
    content_width: f32,
}

impl DocumentGeometry {
    pub fn reset(&mut self, generation: Generation) {
        if generation == self.generation {
            return;
        }

        self.generation = generation;
        self.blocks.clear();
        self.line_owners.clear();
    }

    pub fn set_content_width(&mut self, width: f32) {
        self.content_width = width;
    }

    pub fn content_width(&self) -> f32 {
        self.content_width
    }

    pub fn gutter_right(&self) -> f32 {
        self.blocks
            .iter()
            .map(BlockGeometry::gutter_right)
            .fold(0.0, f32::max)
    }

    pub fn register(
        &mut self,
        generation: Generation,
        source: BlockSource,
        lines: Vec<VisualLine>,
    ) {
        self.reset(generation);

        let block = BlockGeometry::new(source, lines);

        let block_id = block.source.block();

        match self
            .blocks
            .iter()
            .position(|entry| entry.source.block() == block_id)
        {
            Some(index) => {
                for line in &self.blocks[index].lines {
                    if self.line_owners.get(&line.source.start) == Some(&block_id) {
                        self.line_owners.remove(&line.source.start);
                    }
                }
                self.blocks[index] = block;
            }
            None => {
                self.blocks.push(block);
                self.blocks.sort_by_key(|entry| entry.source.block());
            }
        }

        let index = self
            .blocks
            .iter()
            .position(|entry| entry.source.block() == block_id)
            .expect("registered block");

        for line in &self.blocks[index].lines {
            self.line_owners
                .entry(line.source.start)
                .and_modify(|owner| *owner = (*owner).min(block_id))
                .or_insert(block_id);
        }
    }

    pub fn set_origin(&mut self, block: usize, origin: Point) {
        let Ok(index) = self
            .blocks
            .binary_search_by_key(&block, |entry| entry.source.block())
        else {
            return;
        };

        self.blocks[index].origin = origin;
    }

    pub fn line_rects(
        &self,
        block: usize,
        lines: Range<usize>,
        height: f32,
        content_width: f32,
    ) -> Vec<Rectangle> {
        let Some(entry) = self.block(block) else {
            return Vec::new();
        };
        let source = entry
            .source
            .index()
            .range(lines.start, lines.end.saturating_sub(1));
        let width =
            content_width.max(entry.origin.x + entry.lines.first().map_or(0.0, |line| line.left));

        entry
            .lines
            .iter()
            .filter(|line| line.source.start < source.end && line.source.end > source.start)
            .filter(|line| !self.earlier_overlap(block, line))
            .map(|line| entry.rect_for_line(line, height, width))
            .collect()
    }

    fn earlier_overlap(&self, block: usize, line: &VisualLine) -> bool {
        self.line_owners
            .get(&line.source.start)
            .is_some_and(|owner| *owner < block)
    }

    pub fn block(&self, block: usize) -> Option<&BlockGeometry> {
        self.blocks
            .binary_search_by_key(&block, |entry| entry.source.block())
            .ok()
            .map(|index| &self.blocks[index])
    }

    pub fn source(&self, block: usize) -> Option<&BlockSource> {
        self.block(block).map(BlockGeometry::source)
    }

    pub fn copy_index(&self) -> Option<&SourceIndex> {
        self.blocks
            .first()
            .map(|block| block.source.copy_index().as_ref())
    }

    pub fn hit(&self, point: Point) -> Option<GeometryHit> {
        let (block, line) = self.nearest(point)?;

        Some(self.hit_in(block, line, point))
    }

    pub fn hit_exact(&self, point: Point) -> Option<GeometryHit> {
        let (block, line) = self.nearest(point)?;
        let entry = &self.blocks[block];
        let visual = &entry.lines[line];
        let top = entry.origin.y + visual.top;
        let bottom = top + visual.height;

        if point.y < top || point.y > bottom {
            return None;
        }

        if self.content_width > 0.0 && point.x > self.content_width {
            return None;
        }

        Some(self.hit_in(block, line, point))
    }

    fn nearest(&self, point: Point) -> Option<(usize, usize)> {
        let mut best: Option<(f32, f32, usize, usize)> = None;

        for (block_index, block) in self.blocks.iter().enumerate() {
            for (line_index, line) in block.lines.iter().enumerate() {
                let top = block.origin.y + line.top;
                let bottom = top + line.height;
                let vertical = match point.y < top {
                    true => top - point.y,
                    false if point.y > bottom => point.y - bottom,
                    false => 0.0,
                };
                let left = block.origin.x + line.left;
                let right = block.origin.x
                    + line
                        .glyphs
                        .last()
                        .map_or(line.left, |glyph| glyph.x + glyph.width);
                let horizontal = match point.x < left {
                    true => left - point.x,
                    false if point.x > right => point.x - right,
                    false => 0.0,
                };

                if best.is_none_or(|(current_vertical, current_horizontal, _, _)| {
                    (vertical, horizontal) < (current_vertical, current_horizontal)
                }) {
                    best = Some((vertical, horizontal, block_index, line_index));
                }
            }
        }

        best.map(|(_, _, block, line)| (block, line))
    }

    fn hit_in(&self, block: usize, line: usize, point: Point) -> GeometryHit {
        let entry = &self.blocks[block];
        let visual = &entry.lines[line];
        let left = entry.origin.x + visual.left;
        let right = entry.origin.x
            + visual
                .glyphs
                .last()
                .map_or(visual.left, |glyph| glyph.x + glyph.width);

        let source_line = entry.source.index().line_of(visual.source.start);

        if point.x < left {
            return GeometryHit {
                block: entry.source.block(),
                line: source_line,
                byte: visual.logical.start,
                gutter: true,
            };
        }

        if point.x >= right {
            return GeometryHit {
                block: entry.source.block(),
                line: source_line,
                byte: visual.logical.end,
                gutter: false,
            };
        }

        let byte = visual
            .glyphs
            .iter()
            .find(|glyph| {
                let x = entry.origin.x + glyph.x;
                point.x >= x && point.x < x + glyph.width
            })
            .map(|glyph| {
                let start = entry.origin.x + glyph.x;
                let fraction = ((point.x - start) / glyph.width.max(f32::EPSILON)).clamp(0.0, 1.0);
                let length = glyph.end.saturating_sub(glyph.start);
                glyph.start + (length as f32 * fraction) as usize
            })
            .unwrap_or(visual.logical.start);

        GeometryHit {
            block: entry.source.block(),
            line: source_line,
            byte,
            gutter: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::markdown::SourceIndex;

    fn source() -> BlockSource {
        let text = crate::markdown::parse("alpha\nbeta")
            .into_iter()
            .find_map(|item| match item {
                crate::markdown::Item::Paragraph(text) => Some(text),
                _ => None,
            })
            .expect("paragraph");
        BlockSource::from_text(0, Arc::new(SourceIndex::new("alpha\nbeta")), &text)
    }

    fn geometry() -> DocumentGeometry {
        let mut geometry = DocumentGeometry::default();
        geometry.set_content_width(200.0);
        let packet = source();
        let index = packet.index().clone();
        geometry.register(
            Generation::new(),
            packet,
            vec![
                VisualLine {
                    top: 0.0,
                    height: 20.0,
                    left: 0.0,
                    logical: 0..5,
                    source: index.range(0, 0),
                    glyphs: vec![Glyph {
                        x: 0.0,
                        width: 50.0,
                        start: 0,
                        end: 5,
                    }],
                },
                VisualLine {
                    top: 20.0,
                    height: 20.0,
                    left: 0.0,
                    logical: 6..10,
                    source: index.range(1, 1),
                    glyphs: vec![Glyph {
                        x: 0.0,
                        width: 40.0,
                        start: 6,
                        end: 10,
                    }],
                },
            ],
        );
        geometry.set_origin(0, Point::new(24.0, 10.0));
        geometry
    }

    #[test]
    fn hit_inside_line_maps_to_byte() {
        let geometry = geometry();
        let hit = geometry.hit(Point::new(24.0 + 25.0, 20.0)).expect("hit");

        assert!(!hit.gutter);
        assert_eq!(hit.block, 0);
        assert_eq!(hit.line, 0);
        assert!(hit.byte > 0 && hit.byte < 5);
    }

    #[test]
    fn hit_left_of_text_is_gutter() {
        let geometry = geometry();
        let hit = geometry.hit(Point::new(10.0, 20.0)).expect("hit");

        assert!(hit.gutter);
        assert_eq!(hit.byte, 0);
    }

    #[test]
    fn nearest_line_clamps_between_blocks() {
        let geometry = geometry();
        let hit = geometry.hit(Point::new(100.0, 1000.0)).expect("hit");

        assert_eq!(hit.line, 1);
        assert_eq!(hit.byte, 10);
    }

    #[test]
    fn hit_above_first_block_clamps_to_first_line() {
        let geometry = geometry();
        let hit = geometry.hit(Point::new(100.0, -100.0)).expect("hit");

        assert_eq!(hit.line, 0);
        assert_eq!(hit.byte, 5);
    }

    #[test]
    fn line_rects_use_configured_height_and_content_width() {
        let geometry = geometry();
        let block = geometry.block(0).expect("block");
        let rects = block.line_rects(0..1, 22.0, geometry.content_width());

        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].height, 22.0);
        assert_eq!(rects[0].width, geometry.content_width());
        assert_eq!(rects[0].x, -24.0);
    }

    #[test]
    fn text_rects_cover_partial_visual_lines() {
        let geometry = geometry();
        let block = geometry.block(0).expect("block");
        let rects = block.text_rects(2..8, 22.0);

        assert_eq!(rects.len(), 2);
        assert!(
            rects
                .iter()
                .all(|rect| rect.height == 22.0 && rect.width > 0.0)
        );
        assert!(rects[0].x > 0.0);
    }

    #[test]
    fn hit_exact_rejects_vertical_gap_but_hit_clamps_it() {
        let mut geometry = geometry();
        geometry.set_content_width(200.0);
        let point = Point::new(24.0 + 10.0, 1000.0);

        assert!(geometry.hit_exact(point).is_none());
        assert!(geometry.hit(point).is_some());
    }

    #[test]
    fn table_cells_on_one_row_share_source_line_and_full_width_rects() {
        let mut geometry = DocumentGeometry::default();
        geometry.set_content_width(300.0);
        let index = Arc::new(SourceIndex::new("| a | b |"));
        let left = BlockSource::from_text(
            0,
            Arc::clone(&index),
            &crate::markdown::parse("a")
                .into_iter()
                .find_map(|item| match item {
                    crate::markdown::Item::Paragraph(text) => Some(text),
                    _ => None,
                })
                .expect("left"),
        );
        let right = BlockSource::from_text(
            4,
            Arc::clone(&index),
            &crate::markdown::parse("b")
                .into_iter()
                .find_map(|item| match item {
                    crate::markdown::Item::Paragraph(text) => Some(text),
                    _ => None,
                })
                .expect("right"),
        );
        let line = |left_x: f32| VisualLine {
            top: 0.0,
            height: 20.0,
            left: left_x,
            logical: 0..1,
            source: 0..9,
            glyphs: vec![Glyph {
                x: left_x,
                width: 10.0,
                start: 0,
                end: 1,
            }],
        };
        geometry.register(Generation::default(), left, vec![line(0.0)]);
        geometry.register(Generation::default(), right, vec![line(0.0)]);
        geometry.set_origin(0, Point::new(20.0, 10.0));
        geometry.set_origin(4, Point::new(120.0, 10.0));

        let left_rects = geometry.line_rects(0, 0..1, 22.0, 300.0);
        let right_rects = geometry.line_rects(4, 0..1, 22.0, 300.0);

        assert_eq!(left_rects.len(), 1);
        assert_eq!(right_rects.len(), 0);
        assert_eq!(left_rects[0].width, 300.0);
        let left_hit = geometry.hit(Point::new(30.0, 15.0)).expect("left hit");
        let right_hit = geometry.hit(Point::new(125.0, 15.0)).expect("right hit");
        assert_eq!(left_hit.block, 0);
        assert_eq!(right_hit.block, 4);
    }

    #[test]
    fn gutter_right_is_maximum_line_start() {
        let geometry = geometry();

        assert_eq!(geometry.gutter_right(), 24.0);
    }

    #[test]
    fn stale_generation_is_not_hit_tested() {
        let mut geometry = geometry();
        let mut next = Generation::new();
        next.advance();
        geometry.reset(next);

        assert!(geometry.hit(Point::new(24.0, 20.0)).is_none());
    }
}
