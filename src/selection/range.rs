use iced::advanced::graphics::text::cosmic_text::{Buffer, Cursor};
use iced::{Point, Rectangle, Size};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Selection {
    anchor: Option<Cursor>,
    focus: Option<Cursor>,
}

impl Selection {
    pub fn new(anchor: Cursor, focus: Cursor) -> Self {
        Self {
            anchor: Some(anchor),
            focus: Some(focus),
        }
    }

    pub fn extend(&mut self, cursor: Cursor) {
        if self.anchor.is_some() {
            self.focus = Some(cursor);
        }
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.focus = None;
    }

    pub fn range(&self) -> Option<(Cursor, Cursor)> {
        let anchor = self.anchor?;
        let focus = self.focus?;

        if anchor == focus {
            return None;
        }

        if anchor < focus {
            Some((anchor, focus))
        } else {
            Some((focus, anchor))
        }
    }

    pub fn text(&self, buffer: &Buffer) -> Option<String> {
        let (start, end) = self.range()?;
        let mut text = String::new();

        for line in start.line..=end.line {
            let Some(buffer_line) = buffer.lines.get(line) else {
                break;
            };

            let line_text = buffer_line.text();
            let from = if line == start.line {
                start.index.min(line_text.len())
            } else {
                0
            };
            let to = if line == end.line {
                end.index.min(line_text.len())
            } else {
                line_text.len()
            };

            if let Some(fragment) = line_text.get(from..to) {
                text.push_str(fragment);
            }

            if line < end.line {
                text.push('\n');
            }
        }

        if text.is_empty() { None } else { Some(text) }
    }

    pub fn bounds(&self, buffer: &Buffer) -> Vec<Rectangle> {
        let Some((start, end)) = self.range() else {
            return Vec::new();
        };

        buffer
            .layout_runs()
            .filter_map(|run| {
                let (x, width) = run.highlight(start, end)?;

                if width <= 0.0 {
                    return None;
                }

                Some(Rectangle::new(
                    Point::new(x, run.line_top),
                    Size::new(width, run.line_height),
                ))
            })
            .collect()
    }
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
    fn multiline_range_has_bounds_for_each_line() {
        let buffer = buffer("ab\ncd");

        let selection = Selection::new(Cursor::new(0, 1), Cursor::new(1, 1));

        let bounds = selection.bounds(&buffer);

        assert!(bounds.len() >= 2);
    }

    #[test]
    fn selection_produces_highlight_bounds() {
        let buffer = buffer("Hello");
        let selection = Selection::new(Cursor::new(0, 0), Cursor::new(0, 5));

        let bounds = selection.bounds(&buffer);

        assert!(!bounds.is_empty());
        assert!(bounds.iter().all(|bounds| bounds.width > 0.0));
    }
}
