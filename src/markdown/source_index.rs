use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLine {
    start: usize,
    content_end: usize,
    end: usize,
}

impl SourceLine {
    pub fn start(self) -> usize {
        self.start
    }

    pub fn content_end(self) -> usize {
        self.content_end
    }

    pub fn end(self) -> usize {
        self.end
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceIndex {
    source: Arc<str>,
    lines: Vec<SourceLine>,
}

impl SourceIndex {
    pub fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut lines = Vec::new();
        let mut start = 0;

        for (index, byte) in bytes.iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }

            let content_end = match index > start && bytes[index - 1] == b'\r' {
                true => index - 1,
                false => index,
            };
            lines.push(SourceLine {
                start,
                content_end,
                end: index + 1,
            });
            start = index + 1;
        }

        if start < bytes.len() || lines.is_empty() && !bytes.is_empty() {
            lines.push(SourceLine {
                start,
                content_end: bytes.len(),
                end: bytes.len(),
            });
        }

        Self {
            source: Arc::from(source),
            lines,
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, index: usize) -> Option<SourceLine> {
        self.lines.get(index).copied()
    }

    pub fn line_of(&self, byte: usize) -> usize {
        if self.lines.is_empty() {
            return 0;
        }

        self.lines
            .iter()
            .position(|line| byte < line.end)
            .unwrap_or(self.lines.len() - 1)
    }

    pub fn range(&self, from: usize, to: usize) -> Range<usize> {
        if self.lines.is_empty() {
            return 0..0;
        }

        let from = from.min(to).min(self.lines.len() - 1);
        let to = to.max(from).min(self.lines.len() - 1);

        self.lines[from].start..self.lines[to].end
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn slice(&self, range: Range<usize>) -> Option<&str> {
        self.source.get(range)
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_has_no_lines() {
        let index = SourceIndex::new("");

        assert_eq!(index.line_count(), 0);
        assert_eq!(index.range(0, 0), 0..0);
    }

    #[test]
    fn source_without_final_newline_has_last_line() {
        let index = SourceIndex::new("first\nsecond");

        assert_eq!(index.line_count(), 2);
        assert_eq!(index.line(0).map(SourceLine::end), Some(6));
        assert_eq!(index.line(1).map(SourceLine::content_end), Some(12));
        assert_eq!(index.range(1, 1), 6..12);
    }

    #[test]
    fn final_newline_does_not_create_phantom_line() {
        let index = SourceIndex::new("one\n");

        assert_eq!(index.line_count(), 1);
        assert_eq!(index.range(0, 0), 0..4);
    }

    #[test]
    fn crlf_keeps_both_bytes_in_line_range() {
        let index = SourceIndex::new("one\r\ntwo\r\n");

        assert_eq!(index.line_count(), 2);
        assert_eq!(index.line(0).map(SourceLine::content_end), Some(3));
        assert_eq!(index.line(0).map(SourceLine::end), Some(5));
        assert_eq!(index.range(0, 1), 0..10);
    }

    #[test]
    fn byte_at_line_boundary_belongs_to_next_line() {
        let index = SourceIndex::new("a\nb");

        assert_eq!(index.line_of(0), 0);
        assert_eq!(index.line_of(1), 0);
        assert_eq!(index.line_of(2), 1);
        assert_eq!(index.line_of(3), 1);
    }
}
