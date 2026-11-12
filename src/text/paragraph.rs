use std::sync::{Arc, RwLockReadGuard, RwLockWriteGuard};

use iced::advanced::graphics;
use iced::advanced::graphics::text::{
    Raw, Version, align, cosmic_text, font_system, to_attributes, to_wrap,
};
use iced::advanced::text::{Alignment, LineHeight, Shaping, Wrapping};
use iced::{Color, Font, Pixels, Point, Rectangle, Size};

use crate::text::Span;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextConfig {
    pub size: Pixels,
    pub font: Font,
    pub line_height: LineHeight,
    pub align_x: Alignment,
    pub align_y: iced::alignment::Vertical,
    pub wrapping: Wrapping,
    pub shaping: Shaping,
}

impl TextConfig {
    pub fn new(
        size: Pixels,
        font: Font,
        line_height: LineHeight,
        wrapping: Wrapping,
        shaping: Shaping,
    ) -> Self {
        Self {
            size,
            font,
            line_height,
            align_x: Alignment::Left,
            align_y: iced::alignment::Vertical::Top,
            wrapping,
            shaping,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difference {
    None,
    Bounds,
    Shape,
}

#[derive(Clone)]
pub struct Paragraph(Arc<Internal>);

#[derive(Clone)]
struct Internal {
    buffer: Arc<cosmic_text::Buffer>,
    font: Font,
    shaping: Shaping,
    wrapping: Wrapping,
    align_x: Alignment,
    align_y: iced::alignment::Vertical,
    bounds: Size,
    min_bounds: Size,
    version: Version,
}

impl Paragraph {
    pub fn new(spans: &[Span<'_>], config: TextConfig, bounds: Size) -> Self {
        let mut font_system = write_font_system();
        let mut buffer = cosmic_text::Buffer::new(
            font_system.raw(),
            cosmic_text::Metrics::new(
                config.size.into(),
                config.line_height.to_absolute(config.size).into(),
            ),
        );

        buffer.set_size(font_system.raw(), Some(bounds.width), Some(bounds.height));
        buffer.set_wrap(font_system.raw(), to_wrap(config.wrapping));
        buffer.set_rich_text(
            font_system.raw(),
            spans.iter().enumerate().map(|(index, span)| {
                let attrs = to_attributes(span.font.unwrap_or(config.font));

                let attrs = match (span.size, span.line_height) {
                    (None, None) => attrs,
                    _ => {
                        let size = span.size.unwrap_or(config.size);

                        attrs.metrics(cosmic_text::Metrics::new(
                            size.into(),
                            span.line_height
                                .unwrap_or(config.line_height)
                                .to_absolute(size)
                                .into(),
                        ))
                    }
                };

                let attrs = match span.color {
                    Some(color) => attrs.color(graphics::text::to_color(color)),
                    None => attrs,
                };

                let attrs = match span.letter_spacing {
                    Some(letter_spacing) => {
                        let size = span.size.unwrap_or(config.size);
                        attrs.letter_spacing(letter_spacing.0 / size.0)
                    }
                    None => attrs,
                };

                (span.text.as_ref(), attrs.metadata(index))
            }),
            &to_attributes(config.font),
            cosmic_text::Shaping::Advanced,
            None,
        );

        let min_bounds = align(&mut buffer, font_system.raw(), config.align_x);

        Self(Arc::new(Internal {
            buffer: Arc::new(buffer),
            font: config.font,
            shaping: config.shaping,
            wrapping: config.wrapping,
            align_x: config.align_x,
            align_y: config.align_y,
            bounds,
            min_bounds,
            version: font_system.version(),
        }))
    }

    pub fn buffer(&self) -> &cosmic_text::Buffer {
        &self.0.buffer
    }

    pub fn resize(&mut self, bounds: Size) {
        let paragraph = Arc::make_mut(&mut self.0);
        let buffer = Arc::make_mut(&mut paragraph.buffer);
        let mut font_system = write_font_system();

        buffer.set_size(font_system.raw(), Some(bounds.width), Some(bounds.height));

        paragraph.min_bounds = align(buffer, font_system.raw(), paragraph.align_x);
        paragraph.bounds = bounds;
    }

    pub fn compare(&self, config: TextConfig, bounds: Size) -> Difference {
        let font_system = read_font_system();
        let paragraph = &self.0;
        let metrics = paragraph.buffer.metrics();

        if paragraph.version != font_system.version()
            || metrics.font_size != config.size.0
            || metrics.line_height != config.line_height.to_absolute(config.size).0
            || paragraph.font != config.font
            || paragraph.shaping != config.shaping
            || paragraph.wrapping != config.wrapping
            || paragraph.align_x != config.align_x
            || paragraph.align_y != config.align_y
        {
            return Difference::Shape;
        }

        if paragraph.bounds != bounds {
            return Difference::Bounds;
        }

        Difference::None
    }

    pub fn min_bounds(&self) -> Size {
        self.0.min_bounds
    }

    pub fn align_x(&self) -> Alignment {
        self.0.align_x
    }

    pub fn align_y(&self) -> iced::alignment::Vertical {
        self.0.align_y
    }

    pub fn hit_span(&self, point: Point) -> Option<usize> {
        let paragraph = &self.0;
        let cursor = paragraph.buffer.hit(point.x, point.y)?;
        let line = paragraph.buffer.lines.get(cursor.line)?;

        if cursor.index >= line.text().len() {
            return None;
        }

        let index = match cursor.affinity {
            cosmic_text::Affinity::Before => cursor.index.saturating_sub(1),
            cosmic_text::Affinity::After => cursor.index,
        };
        let glyphs = line
            .layout_opt()
            .as_ref()?
            .iter()
            .flat_map(|line| line.glyphs.iter());
        let glyph = glyphs
            .into_iter()
            .find(|glyph| glyph.start <= index && index < glyph.end)?;

        Some(glyph.metadata)
    }

    pub fn span_bounds(&self, index: usize) -> Vec<Rectangle> {
        let paragraph = &self.0;
        let mut bounds = Vec::new();
        let mut current_bounds = None;
        let glyphs = paragraph
            .buffer
            .layout_runs()
            .flat_map(|run| {
                let line_top = run.line_top;
                let line_height = run.line_height;

                run.glyphs
                    .iter()
                    .map(move |glyph| (line_top, line_height, glyph))
            })
            .skip_while(|(_, _, glyph)| glyph.metadata != index)
            .take_while(|(_, _, glyph)| glyph.metadata == index);

        for (line_top, line_height, glyph) in glyphs {
            let y = line_top + glyph.y;
            let glyph_bounds = || {
                Rectangle::new(
                    Point::new(glyph.x, y),
                    Size::new(glyph.w, glyph.line_height_opt.unwrap_or(line_height)),
                )
            };

            match current_bounds.as_mut() {
                None => current_bounds = Some(glyph_bounds()),
                Some(current) if y != current.y => {
                    bounds.push(*current);
                    *current = glyph_bounds();
                }
                Some(current) => current.width += glyph.w,
            }
        }

        bounds.extend(current_bounds);
        bounds
    }

    pub fn draw<Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        color: Color,
        viewport: &Rectangle,
    ) where
        Renderer: graphics::text::Renderer,
    {
        let position = bounds.anchor(self.min_bounds(), self.align_x(), self.align_y());

        renderer.fill_raw(Raw {
            buffer: Arc::downgrade(&self.0.buffer),
            position,
            color,
            clip_bounds: *viewport,
        });
    }
}

impl std::fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let paragraph = &self.0;
        let metrics = paragraph.buffer.metrics();

        f.debug_struct("Paragraph")
            .field("font", &paragraph.font)
            .field("shaping", &paragraph.shaping)
            .field("align_x", &paragraph.align_x)
            .field("align_y", &paragraph.align_y)
            .field("bounds", &paragraph.bounds)
            .field("min_bounds", &paragraph.min_bounds)
            .field("metrics", &(metrics.font_size, metrics.line_height))
            .finish()
    }
}

impl Default for Paragraph {
    fn default() -> Self {
        Self(Arc::new(Internal {
            buffer: Arc::new(cosmic_text::Buffer::new_empty(cosmic_text::Metrics {
                font_size: 1.0,
                line_height: 1.0,
            })),
            font: Font::default(),
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
            align_x: Alignment::Default,
            align_y: iced::alignment::Vertical::Top,
            bounds: Size::ZERO,
            min_bounds: Size::ZERO,
            version: Version::default(),
        }))
    }
}

fn write_font_system() -> RwLockWriteGuard<'static, graphics::text::FontSystem> {
    match font_system().write() {
        Ok(font_system) => font_system,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn read_font_system() -> RwLockReadGuard<'static, graphics::text::FontSystem> {
    match font_system().read() {
        Ok(font_system) => font_system,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Pixels;

    use crate::config::{FontChain, Fonts};
    use crate::theme::{FontBook, load_embedded_fonts};

    fn config() -> TextConfig {
        crate::theme::isolate_system_fonts();

        TextConfig::new(
            Pixels(16.0),
            Font::default(),
            LineHeight::Relative(1.3),
            Wrapping::default(),
            Shaping::default(),
        )
    }

    #[test]
    fn letter_spacing_increases_measured_width() {
        let config = config();
        load_embedded_fonts();
        let fonts = Fonts::default();
        let font = FontBook::new(&fonts).default_font(&FontChain::new(vec!["latin".to_owned()]));
        let plain = Span::new("ab").font(font).size(Pixels(10.0));
        let spaced = Span::new("ab")
            .font(font)
            .size(Pixels(10.0))
            .letter_spacing(Pixels(2.0));
        let bounds = Size::new(1_000_000.0, 1_000_000.0);

        let plain_width = Paragraph::new(&[plain], config, bounds).min_bounds().width;
        let spaced_width = Paragraph::new(&[spaced], config, bounds).min_bounds().width;

        assert!(
            spaced_width > plain_width + 1.5,
            "plain={plain_width} spaced={spaced_width}"
        );
    }

    #[test]
    fn compare_reports_bounds_shape_and_none() {
        let config = config();
        let bounds = Size::new(400.0, 200.0);
        let paragraph = Paragraph::new(&[Span::new("word")], config, bounds);
        let wider = Size::new(300.0, 200.0);
        let mut larger = config;
        larger.size = Pixels(20.0);

        assert_eq!(paragraph.compare(config, wider), Difference::Bounds);
        assert_eq!(paragraph.compare(larger, wider), Difference::Shape);
        assert_eq!(paragraph.compare(config, bounds), Difference::None);
    }

    #[test]
    fn draw_anchor_does_not_change_between_calls() {
        let config = config();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(400.0, 200.0));
        let paragraph = Paragraph::new(&[Span::new("word")], config, bounds.size());
        let anchor = |paragraph: &Paragraph| {
            bounds.anchor(
                paragraph.min_bounds(),
                paragraph.align_x(),
                paragraph.align_y(),
            )
        };

        assert_eq!(anchor(&paragraph), anchor(&paragraph));
        assert_eq!(paragraph.align_x(), Alignment::Left);
        assert_eq!(paragraph.align_y(), iced::alignment::Vertical::Top);
    }
}
