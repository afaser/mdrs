use iced::Color;

use crate::config::palette::{Palette, PaletteKey};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorRef {
    Literal(Color),
    Palette(PaletteKey),
}

impl ColorRef {
    pub fn parse(value: &str) -> Option<Self> {
        if value.starts_with('#') {
            return literal(value).map(Self::Literal);
        }

        PaletteKey::parse(value).map(Self::Palette)
    }

    pub fn resolve(self, palette: &Palette) -> Color {
        match self {
            Self::Literal(color) => color,
            Self::Palette(key) => palette.color(key),
        }
    }
}

pub fn literal(value: &str) -> Option<Color> {
    let digits = value.strip_prefix('#')?;

    match digits.len() {
        6 => channel(digits).map(|(r, g, b)| Color::from_rgb8(r, g, b)),
        8 => {
            let (red, green, blue) = channel(digits)?;
            let alpha = alpha(digits)?;
            Some(Color::from_rgba8(red, green, blue, alpha as f32 / 255.0))
        }
        _ => None,
    }
}

fn channel(digits: &str) -> Option<(u8, u8, u8)> {
    let red = u8::from_str_radix(digits.get(0..2)?, 16).ok()?;
    let green = u8::from_str_radix(digits.get(2..4)?, 16).ok()?;
    let blue = u8::from_str_radix(digits.get(4..6)?, 16).ok()?;

    Some((red, green, blue))
}

fn alpha(digits: &str) -> Option<u8> {
    u8::from_str_radix(digits.get(6..8)?, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_parses_rgb_and_rgba() {
        assert_eq!(
            ColorRef::parse("#1e1e2e"),
            Some(ColorRef::Literal(Color::from_rgb8(0x1e, 0x1e, 0x2e)))
        );
        assert_eq!(
            ColorRef::parse("#1e1e2e80"),
            Some(ColorRef::Literal(Color::from_rgba8(
                0x1e,
                0x1e,
                0x2e,
                128.0 / 255.0
            )))
        );
    }

    #[test]
    fn name_parses_as_palette_key() {
        assert_eq!(
            ColorRef::parse("accent"),
            Some(ColorRef::Palette(PaletteKey::Accent))
        );
        assert_eq!(ColorRef::parse("not-a-color"), None);
    }

    #[test]
    fn palette_reference_resolves_after_override() {
        let palette = Palette {
            accent: Color::from_rgb8(0xff, 0x00, 0x00),
            ..Palette::default()
        };

        assert_eq!(
            ColorRef::parse("accent").map(|color| color.resolve(&palette)),
            Some(Color::from_rgb8(0xff, 0x00, 0x00))
        );
    }
}
