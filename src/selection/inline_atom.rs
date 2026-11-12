use std::ops::Range;

use crate::markdown::Uri;
use iced::advanced::text::{LineHeight, Shaping};
use iced::{Background, Border, Color, Font, Padding, Pixels};

use crate::text::Paragraph;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InlineConfig {
    pub size: Pixels,
    pub font: Font,
    pub line_height: LineHeight,
    pub shaping: Shaping,
    pub space: f32,
}

#[derive(Debug, Clone)]
pub(super) struct Chip {
    paragraph: Paragraph,
    content_width: f32,
    content_ascent: f32,
    content_descent: f32,
    padding: Padding,
    background: Background,
    border: Border,
}

#[derive(Debug, Clone)]
pub(super) struct TextBody {
    paragraph: Paragraph,
    width: f32,
    ascent: f32,
    descent: f32,
    styles: Vec<TextStyle>,
}

#[derive(Debug, Clone)]
pub(super) struct TextStyle {
    pub(super) range: Range<usize>,
    pub(super) underline: bool,
    pub(super) strikethrough: bool,
    pub(super) link: Option<Uri>,
    pub(super) color: Option<Color>,
    pub(super) size: f32,
}

#[derive(Debug, Clone)]
pub(super) struct Fragment {
    pub(super) range: Range<usize>,
    pub(super) offset: f32,
    pub(super) width: f32,
}

#[derive(Debug, Clone)]
pub(super) enum AtomBody {
    Text(TextBody),
    Space { width: f32 },
    Chip(Box<Chip>),
}

#[derive(Debug, Clone)]
pub(super) struct Atom {
    pub(super) range: Range<usize>,
    pub(super) body: AtomBody,
}

impl Atom {
    pub(super) fn width(&self) -> f32 {
        match &self.body {
            AtomBody::Text(text) => text.width,
            AtomBody::Space { width } => *width,
            AtomBody::Chip(chip) => chip.width(),
        }
    }

    pub(super) fn ascent(&self) -> f32 {
        match &self.body {
            AtomBody::Text(text) => text.ascent,
            AtomBody::Space { .. } => 0.0,
            AtomBody::Chip(chip) => chip.content_ascent + chip.padding.top,
        }
    }

    pub(super) fn descent(&self) -> f32 {
        match &self.body {
            AtomBody::Text(text) => text.descent,
            AtomBody::Space { .. } => 0.0,
            AtomBody::Chip(chip) => chip.content_descent + chip.padding.bottom,
        }
    }

    pub(super) fn is_space(&self) -> bool {
        matches!(self.body, AtomBody::Space { .. })
    }

    #[cfg(test)]
    pub(super) fn is_chip(&self) -> bool {
        matches!(self.body, AtomBody::Chip(_))
    }

    pub(super) fn text(&self) -> Option<&TextBody> {
        match &self.body {
            AtomBody::Text(text) => Some(text),
            AtomBody::Space { .. } | AtomBody::Chip(_) => None,
        }
    }

    pub(super) fn chip(&self) -> Option<&Chip> {
        match &self.body {
            AtomBody::Chip(chip) => Some(chip.as_ref()),
            AtomBody::Text(_) | AtomBody::Space { .. } => None,
        }
    }

    pub(super) fn content_len(&self) -> usize {
        self.range.len()
    }

    pub(super) fn content_width(&self) -> f32 {
        match &self.body {
            AtomBody::Text(text) => text.width,
            AtomBody::Space { .. } => 0.0,
            AtomBody::Chip(chip) => chip.content_width,
        }
    }

    pub(super) fn content_ascent(&self) -> f32 {
        match &self.body {
            AtomBody::Text(text) => text.ascent,
            AtomBody::Space { .. } => 0.0,
            AtomBody::Chip(chip) => chip.content_ascent,
        }
    }

    pub(super) fn content_descent(&self) -> f32 {
        match &self.body {
            AtomBody::Text(text) => text.descent,
            AtomBody::Space { .. } => 0.0,
            AtomBody::Chip(chip) => chip.content_descent,
        }
    }

    pub(super) fn content_offset(&self) -> f32 {
        match &self.body {
            AtomBody::Text(_) | AtomBody::Space { .. } => 0.0,
            AtomBody::Chip(chip) => chip.padding.left,
        }
    }

    pub(super) fn paragraph(&self) -> Option<&Paragraph> {
        match &self.body {
            AtomBody::Text(text) => Some(&text.paragraph),
            AtomBody::Space { .. } => None,
            AtomBody::Chip(chip) => Some(&chip.paragraph),
        }
    }

    pub(super) fn fragments(&self, max_width: f32) -> Option<Vec<Fragment>> {
        let text = self.text()?;

        if self.width() <= max_width || max_width <= 0.0 {
            return None;
        }

        let run = text.paragraph.buffer().layout_runs().next()?;
        let mut fragments = Vec::new();
        let mut start_byte = 0;
        let mut start_x = 0.0;

        for glyph in run.glyphs {
            let overflow = glyph.x + glyph.w - start_x > max_width;
            let has_content = glyph.start > start_byte;

            if !overflow || !has_content {
                continue;
            }

            fragments.push(Fragment {
                range: start_byte..glyph.start,
                offset: start_x,
                width: glyph.x - start_x,
            });
            start_byte = glyph.start;
            start_x = glyph.x;
        }

        if start_byte < self.content_len() {
            fragments.push(Fragment {
                range: start_byte..self.content_len(),
                offset: start_x,
                width: self.width() - start_x,
            });
        }

        (!fragments.is_empty()).then_some(fragments)
    }
}

impl Chip {
    fn width(&self) -> f32 {
        self.content_width + self.padding.x()
    }

    pub(super) fn content_width(&self) -> f32 {
        self.content_width
    }

    pub(super) fn content_ascent(&self) -> f32 {
        self.content_ascent
    }

    pub(super) fn content_descent(&self) -> f32 {
        self.content_descent
    }

    pub(super) fn padding(&self) -> Padding {
        self.padding
    }

    pub(super) fn background(&self) -> Background {
        self.background
    }

    pub(super) fn border(&self) -> Border {
        self.border
    }
}

impl TextBody {
    pub(super) fn paragraph(&self) -> &Paragraph {
        &self.paragraph
    }

    pub(super) fn styles(&self) -> &[TextStyle] {
        &self.styles
    }
}

impl TextBody {
    pub(super) fn new(
        paragraph: Paragraph,
        width: f32,
        ascent: f32,
        descent: f32,
        styles: Vec<TextStyle>,
    ) -> Self {
        Self {
            paragraph,
            width,
            ascent,
            descent,
            styles,
        }
    }
}

impl Chip {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        paragraph: Paragraph,
        content_width: f32,
        content_ascent: f32,
        content_descent: f32,
        padding: Padding,
        background: Background,
        border: Border,
    ) -> Self {
        Self {
            paragraph,
            content_width,
            content_ascent,
            content_descent,
            padding,
            background,
            border,
        }
    }
}
