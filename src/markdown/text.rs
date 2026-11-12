use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::sync::Arc;

use iced::font::{self, Font};

use crate::markdown::settings::Style;
use crate::markdown::uri::Uri;
use crate::text::Span as StyledSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSegment {
    pub logical: Range<usize>,
    pub source: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Text {
    source: Range<usize>,
    spans: Vec<Span>,
    last_style: Cell<Option<Style>>,
    last_styled_spans: RefCell<Arc<[StyledSpan<'static>]>>,
}

impl Text {
    pub(super) fn new(spans: Vec<Span>) -> Self {
        let source = match (spans.first(), spans.last()) {
            (Some(first), Some(last)) => first.source.start..last.source.end,
            _ => 0..0,
        };

        Self {
            source,
            spans,
            last_style: Cell::default(),
            last_styled_spans: RefCell::default(),
        }
    }

    pub fn source(&self) -> Range<usize> {
        self.source.clone()
    }

    pub fn source_segments(&self) -> Vec<SourceSegment> {
        let mut segments = Vec::with_capacity(self.spans.len());
        let mut logical_start = 0;

        for span in &self.spans {
            let logical_end = logical_start + span.text.len();
            segments.push(SourceSegment {
                logical: logical_start..logical_end,
                source: span.source.clone(),
            });
            logical_start = logical_end;
        }

        segments
    }

    pub fn spans(&self, style: Style) -> Arc<[StyledSpan<'static>]> {
        if Some(style) != self.last_style.get() {
            *self.last_styled_spans.borrow_mut() =
                self.spans.iter().map(|span| span.view(&style)).collect();
            self.last_style.set(Some(style));
        }

        self.last_styled_spans.borrow().clone()
    }
}

#[derive(Debug, Clone)]
pub(super) struct Span {
    pub(super) text: String,
    pub(super) source: Range<usize>,
    pub(super) strong: bool,
    pub(super) emphasis: bool,
    pub(super) strikethrough: bool,
    pub(super) code: bool,
    pub(super) link: Option<Uri>,
}

impl Span {
    fn view(&self, style: &Style) -> StyledSpan<'static> {
        let span = StyledSpan::new(self.text.clone()).strikethrough(self.strikethrough);
        let emphasized = self.strong || self.emphasis;
        let span = match (self.code, emphasized) {
            (true, _) => span
                .font(style.inline_code_font)
                .color(style.inline_code_color)
                .background(style.inline_code_highlight.background)
                .border(style.inline_code_highlight.border)
                .padding(style.inline_code_padding),
            (false, true) => span.font(Font {
                weight: match self.strong {
                    true => font::Weight::Bold,
                    false => font::Weight::Normal,
                },
                style: match self.emphasis {
                    true => font::Style::Italic,
                    false => font::Style::Normal,
                },
                ..style.font
            }),
            (false, false) => span.font(style.font),
        };

        match &self.link {
            Some(link) => span.color(style.link_color).link(link.clone()),
            None => span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{Background, Border, Color, Font, Padding};

    use crate::text::Highlight;

    fn style() -> Style {
        Style {
            font: Font::default(),
            inline_code_highlight: Highlight {
                background: Background::Color(Color::BLACK),
                border: Border::default(),
            },
            inline_code_padding: Padding::default(),
            inline_code_color: Color::WHITE,
            inline_code_font: Font::MONOSPACE,
            code_block_font: Font::MONOSPACE,
            link_color: Color::from_rgb8(0, 0, 255),
        }
    }

    fn text() -> Text {
        Text::new(vec![Span {
            text: "alpha".to_owned(),
            source: 0..5,
            strong: false,
            emphasis: false,
            strikethrough: false,
            code: false,
            link: None,
        }])
    }

    #[test]
    fn styled_spans_are_cached_per_style() {
        let text = text();
        let first = text.spans(style());
        let second = text.spans(style());

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first[0].size, None);
        assert_eq!(first[0].text.as_ref(), "alpha");
    }

    #[test]
    fn empty_text_has_empty_source() {
        let text = Text::new(Vec::new());

        assert_eq!(text.source(), 0..0);
        assert!(text.source_segments().is_empty());
    }

    #[test]
    fn inline_code_uses_style_highlight() {
        let style = style();
        let text = Text::new(vec![Span {
            text: "code".to_owned(),
            source: 2..6,
            strong: false,
            emphasis: false,
            strikethrough: false,
            code: true,
            link: None,
        }]);
        let spans = text.spans(style);

        assert_eq!(spans[0].font, Some(style.inline_code_font));
        assert_eq!(spans[0].highlight, Some(style.inline_code_highlight));
    }
}
