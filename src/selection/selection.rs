#![allow(clippy::module_inception)]

use iced::advanced::graphics::text::cosmic_text::{Buffer, Cursor, LayoutRun};
use iced::{Point, Rectangle, Size};

#[derive(Debug, Clone, Copy, Default)]
pub struct Selection {
    anchor: Option<Cursor>,
    focus: Option<Cursor>,
}

#[derive(Debug, Clone, Copy)]
struct SelectionRange {
    from: Cursor,
    to: Cursor,
}

impl SelectionRange {
    fn new(anchor: Cursor, focus: Cursor) -> Option<Self> {
        if anchor == focus {
            return None;
        }

        Some(Self {
            from: anchor.min(focus),
            to: anchor.max(focus),
        })
    }
}

impl Selection {
    pub fn new(anchor: Cursor, focus: Cursor) -> Self {
        Self {
            anchor: Some(anchor),
            focus: Some(focus),
        }
    }

    fn range(&self) -> Option<SelectionRange> {
        SelectionRange::new(self.anchor?, self.focus?)
    }

    #[cfg(test)]
    pub fn text(&self, buffer: &Buffer) -> Option<String> {
        let range = self.range()?;
        let mut text = String::new();

        for line in range.from.line..=range.to.line {
            let Some(buffer_line) = buffer.lines.get(line) else {
                break;
            };

            let line_text = buffer_line.text();
            let from = line_from(line, range, line_text.len());
            let to = line_to(line, range, line_text.len());

            if let Some(fragment) = line_text.get(from..to) {
                text.push_str(fragment);
            }

            if line < range.to.line {
                text.push('\n');
            }
        }

        (!text.is_empty()).then_some(text)
    }

    pub fn bounds(&self, buffer: &Buffer) -> Vec<Rectangle> {
        let Some(range) = self.range() else {
            return Vec::new();
        };

        buffer
            .layout_runs()
            .filter_map(|run| run_rect(&run, range))
            .collect()
    }
}

#[cfg(test)]
fn line_from(line: usize, range: SelectionRange, length: usize) -> usize {
    if line != range.from.line {
        return 0;
    }

    range.from.index.min(length)
}

#[cfg(test)]
fn line_to(line: usize, range: SelectionRange, length: usize) -> usize {
    if line != range.to.line {
        return length;
    }

    range.to.index.min(length)
}

fn run_rect(run: &LayoutRun<'_>, range: SelectionRange) -> Option<Rectangle> {
    let (x, width) = run.highlight(range.from, range.to)?;

    if !has_area(width) {
        return None;
    }

    Some(Rectangle::new(
        Point::new(x, run.line_top),
        Size::new(width, run.line_height),
    ))
}

fn has_area(width: f32) -> bool {
    width > f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Font;
    use iced::advanced::graphics::text::cosmic_text::{Metrics, Shaping};
    use iced::advanced::graphics::text::{font_system, to_attributes};

    fn buffer(text: &str) -> Buffer {
        crate::theme::isolate_system_fonts();
        let mut font_system = font_system().write().expect("font system");
        let mut buffer = Buffer::new(font_system.raw(), Metrics::new(16.0, 20.0));

        buffer.set_size(font_system.raw(), Some(400.0), Some(200.0));
        buffer.set_text(
            font_system.raw(),
            text,
            &to_attributes(Font::default()),
            Shaping::Advanced,
            None,
        );

        buffer
    }

    #[test]
    fn forward_range_copies_text() {
        let buffer = buffer("Hello");
        let selection = Selection::new(Cursor::new(0, 0), Cursor::new(0, 5));

        assert_eq!(selection.text(&buffer).as_deref(), Some("Hello"));
    }

    #[test]
    fn reverse_range_copies_same_text() {
        let buffer = buffer("Hello");
        let selection = Selection::new(Cursor::new(0, 5), Cursor::new(0, 0));

        assert_eq!(selection.text(&buffer).as_deref(), Some("Hello"));
    }

    #[test]
    fn multiline_range_keeps_newlines() {
        let buffer = buffer("ab\ncd");
        let selection = Selection::new(Cursor::new(0, 1), Cursor::new(1, 1));

        assert_eq!(selection.text(&buffer).as_deref(), Some("b\nc"));
    }

    #[test]
    fn empty_range_copies_nothing() {
        let buffer = buffer("Hello");
        let selection = Selection::new(Cursor::new(0, 2), Cursor::new(0, 2));

        assert!(selection.text(&buffer).is_none());
    }

    #[test]
    fn spaces_are_preserved() {
        let buffer = buffer("a  b");
        let selection = Selection::new(Cursor::new(0, 1), Cursor::new(0, 4));

        assert_eq!(selection.text(&buffer).as_deref(), Some("  b"));
    }

    #[test]
    fn single_line_range_does_not_cross_line_boundary() {
        let buffer = buffer("ab\ncd");
        let selection = Selection::new(Cursor::new(0, 0), Cursor::new(0, 2));

        assert_eq!(selection.text(&buffer).as_deref(), Some("ab"));
    }

    #[test]
    fn selection_bounds_cover_each_visual_line() {
        let buffer = buffer("ab\ncd");
        let selection = Selection::new(Cursor::new(0, 1), Cursor::new(1, 1));

        let bounds = selection.bounds(&buffer);

        assert!(bounds.len() >= 2);
        assert!(bounds.iter().all(|bounds| bounds.width > 0.0));
    }
}
