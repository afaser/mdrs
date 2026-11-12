use std::cell::RefCell;
use std::collections::HashMap;

use iced::advanced::graphics::text::{font_system, to_attributes};
use iced::{Font, Pixels};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    ascent: f32,
    descent: f32,
    space: f32,
}

impl FontMetrics {
    pub fn of(font: Font, size: Pixels) -> Option<Self> {
        let mut font_system = font_system().write().ok()?;
        let attrs = to_attributes(font);
        let raw = font_system.raw();
        let id = {
            let db = raw.db_mut();
            let mut best = None;

            for face in db.faces() {
                let family = match attrs.family {
                    iced::advanced::graphics::text::cosmic_text::Family::Name(name) => {
                        face.families.iter().any(|(family, _)| family == name)
                    }
                    _ => true,
                };
                let style = face.style == attrs.style;
                let stretch = face.stretch == attrs.stretch;
                let score = face.weight.0.abs_diff(attrs.weight.0)
                    + if style { 0 } else { 1_000 }
                    + if stretch { 0 } else { 100 };
                let score = if family { score } else { score + 10_000 };

                if best.is_none_or(|(_, current)| score < current) {
                    best = Some((face.id, score));
                }
            }

            best.map(|(id, _)| id)?
        };
        let face = raw.get_font(id, attrs.weight)?;
        let swash = face.as_swash();
        let metrics = swash.metrics(&[]);
        let upem = f32::from(metrics.units_per_em);

        if upem <= f32::EPSILON {
            return None;
        }

        let scale = size.0 / upem;
        let space_glyph = swash.charmap().map(' ');

        if space_glyph == 0 {
            return None;
        }

        let space = swash.glyph_metrics(&[]).advance_width(space_glyph) * scale;

        Some(Self {
            ascent: metrics.ascent * scale,
            descent: metrics.descent.abs() * scale,
            space,
        })
    }

    pub fn ascent(self) -> f32 {
        self.ascent
    }

    pub fn descent(self) -> f32 {
        self.descent
    }

    pub fn space(self) -> f32 {
        self.space
    }
}

#[derive(Default)]
pub struct FontMetricsBook {
    cache: RefCell<HashMap<(Font, u32), Option<FontMetrics>>>,
}

impl FontMetricsBook {
    pub fn metrics(&self, font: Font, size: Pixels) -> Option<FontMetrics> {
        let key = (font, size.0.to_bits());

        if let Some(metrics) = self.cache.borrow().get(&key) {
            return *metrics;
        }

        let metrics = FontMetrics::of(font, size);
        self.cache.borrow_mut().insert(key, metrics);
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FontChain, Fonts};
    use crate::theme::font::FontBook;

    fn code_font() -> Font {
        crate::theme::load_embedded_fonts();
        let book = FontBook::new(&Fonts::default());
        book.chain_font(
            &FontChain::new(vec!["code".to_owned()]),
            iced::font::Weight::Normal,
            iced::font::Style::Normal,
        )
        .expect("code font")
    }

    #[test]
    fn embedded_code_font_has_positive_metrics() {
        let metrics = FontMetrics::of(code_font(), Pixels(13.0)).expect("metrics");

        assert!(metrics.ascent() > 0.0);
        assert!(metrics.descent() > 0.0);
        assert!(metrics.space() > 0.0);
    }

    #[test]
    fn metrics_cache_returns_same_values() {
        let book = FontMetricsBook::default();
        let first = book.metrics(code_font(), Pixels(13.0));
        let second = book.metrics(code_font(), Pixels(13.0));

        assert_eq!(first, second);
    }
}
