use std::borrow::Cow;

use iced::advanced::text::LineHeight;
use iced::{Background, Border, Color, Font, Padding, Pixels};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Highlight {
    pub background: Background,
    pub border: Border,
}

#[derive(Debug, Clone)]
pub struct Span<'a> {
    pub text: Cow<'a, str>,
    pub size: Option<Pixels>,
    pub line_height: Option<LineHeight>,
    pub letter_spacing: Option<Pixels>,
    pub font: Option<Font>,
    pub color: Option<Color>,
    pub link: Option<String>,
    pub highlight: Option<Highlight>,
    pub padding: Padding,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Default for Span<'_> {
    fn default() -> Self {
        Self {
            text: Cow::Borrowed(""),
            size: None,
            line_height: None,
            letter_spacing: None,
            font: None,
            color: None,
            link: None,
            highlight: None,
            padding: Padding::default(),
            underline: false,
            strikethrough: false,
        }
    }
}

impl<'a> Span<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = Some(line_height.into());
        self
    }

    pub fn letter_spacing(mut self, letter_spacing: impl Into<Pixels>) -> Self {
        self.letter_spacing = Some(letter_spacing.into());
        self
    }

    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn font_maybe(mut self, font: Option<impl Into<Font>>) -> Self {
        self.font = font.map(Into::into);
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn color_maybe(mut self, color: Option<impl Into<Color>>) -> Self {
        self.color = color.map(Into::into);
        self
    }

    pub fn link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());
        self
    }

    pub fn background(self, background: impl Into<Background>) -> Self {
        let background = background.into();

        match self.highlight {
            Some(highlight) => Self {
                highlight: Some(Highlight {
                    background,
                    ..highlight
                }),
                ..self
            },
            None => Self {
                highlight: Some(Highlight {
                    background,
                    border: Border::default(),
                }),
                ..self
            },
        }
    }

    pub fn border(self, border: impl Into<Border>) -> Self {
        let border = border.into();

        match self.highlight {
            Some(highlight) => Self {
                highlight: Some(Highlight {
                    border,
                    ..highlight
                }),
                ..self
            },
            None => Self {
                highlight: Some(Highlight {
                    background: Background::Color(Color::TRANSPARENT),
                    border,
                }),
                ..self
            },
        }
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.strikethrough = strikethrough;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styled_span_keeps_letter_spacing_and_highlight() {
        let span = Span::new("code".to_owned())
            .size(Pixels(12.0))
            .letter_spacing(Pixels(2.0))
            .color(Color::WHITE)
            .background(Color::BLACK)
            .border(Border::default().width(1.0))
            .padding(Padding::from(3.0));

        assert_eq!(span.text, "code");
        assert_eq!(span.size, Some(Pixels(12.0)));
        assert_eq!(span.letter_spacing, Some(Pixels(2.0)));
        assert_eq!(span.padding, Padding::from(3.0));
        assert_eq!(span.highlight.expect("highlight").border.width, 1.0);
    }
}
