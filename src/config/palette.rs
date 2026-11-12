use iced::Color;

use crate::config::colors::ColorRef;
use crate::config::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteKey {
    Background,
    Text,
    Accent,
    Danger,
    Border,
    Surface,
    Selection,
    Primary,
    Success,
    Warning,
}

impl PaletteKey {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "background" => Some(Self::Background),
            "text" => Some(Self::Text),
            "accent" => Some(Self::Accent),
            "danger" => Some(Self::Danger),
            "border" => Some(Self::Border),
            "surface" => Some(Self::Surface),
            "selection" => Some(Self::Selection),
            "primary" => Some(Self::Primary),
            "success" => Some(Self::Success),
            "warning" => Some(Self::Warning),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub background: Color,
    pub background_opacity: f32,
    pub text: Color,
    pub accent: Color,
    pub danger: Color,
    pub border: Color,
    pub surface: Color,
    pub selection: Color,
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
}

impl Default for Palette {
    fn default() -> Self {
        let danger = Color::from_rgb8(0xf3, 0x8b, 0xa8);
        let accent = Color::from_rgb8(0xa6, 0xe3, 0xa1);

        Self {
            background: Color::from_rgb8(0x1e, 0x1e, 0x2e),
            background_opacity: 1.0,
            text: Color::from_rgb8(0xcd, 0xd6, 0xf4),
            accent,
            danger,
            border: Color::from_rgb8(0x45, 0x47, 0x5a),
            surface: Color::from_rgb8(0x31, 0x32, 0x44),
            selection: Color::from_rgb8(0x45, 0x47, 0x5a),
            primary: accent,
            success: accent,
            warning: danger,
        }
    }
}

impl Palette {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        let mut palette = base;
        palette.background = field(node, "background", base.background, &base);
        palette.background_opacity = node
            .child("background-opacity")
            .and_then(|node| node.number())
            .map(|value| value.clamp(0.0, 1.0))
            .unwrap_or(base.background_opacity);
        palette.text = field(node, "text", base.text, &base);
        palette.accent = field(node, "accent", base.accent, &base);
        palette.danger = field(node, "danger", base.danger, &base);
        palette.border = field(node, "border", base.border, &base);
        palette.surface = field(node, "surface", base.surface, &base);
        palette.selection = field(node, "selection", base.selection, &base);
        palette.primary = field(node, "primary", base.primary, &base);
        palette.success = field(node, "success", base.success, &base);
        palette.warning = field(node, "warning", base.warning, &base);

        palette
    }

    pub fn color(&self, key: PaletteKey) -> Color {
        match key {
            PaletteKey::Background => self.background,
            PaletteKey::Text => self.text,
            PaletteKey::Accent => self.accent,
            PaletteKey::Danger => self.danger,
            PaletteKey::Border => self.border,
            PaletteKey::Surface => self.surface,
            PaletteKey::Selection => self.selection,
            PaletteKey::Primary => self.primary,
            PaletteKey::Success => self.success,
            PaletteKey::Warning => self.warning,
        }
    }

    pub fn background_color(&self) -> Color {
        Color {
            a: self.background.a * self.background_opacity,
            ..self.background
        }
    }
}

fn field(node: Node<'_>, name: &str, base: Color, palette: &Palette) -> Color {
    let Some(value) = node.child(name).and_then(|node| node.text()) else {
        return base;
    };

    match ColorRef::parse(value) {
        Some(ColorRef::Literal(color)) => color,
        Some(ColorRef::Palette(key)) => palette.color(key),
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn default_palette_matches_catppuccin_green() {
        let palette = Palette::default();

        assert_eq!(palette.background, Color::from_rgb8(0x1e, 0x1e, 0x2e));
        assert_eq!(palette.text, Color::from_rgb8(0xcd, 0xd6, 0xf4));
        assert_eq!(palette.accent, Color::from_rgb8(0xa6, 0xe3, 0xa1));
        assert_eq!(palette.danger, Color::from_rgb8(0xf3, 0x8b, 0xa8));
    }

    #[test]
    fn one_color_changes_only_that_field() {
        let document = Document::parse(
            r##"
theme {
    palette {
        accent "#ff0000"
    }
}
"##,
        )
        .expect("document");
        let root = document.root();
        let node = root.child("theme").and_then(|theme| theme.child("palette"));
        let palette = Palette::from_node(Palette::default(), node);

        assert_eq!(palette.accent, Color::from_rgb8(0xff, 0x00, 0x00));
        assert_eq!(palette.background, Palette::default().background);
    }

    #[test]
    fn invalid_color_keeps_default() {
        let document = Document::parse(
            r##"
theme {
    palette {
        background "not-a-color"
        accent "#ff0000"
    }
}
"##,
        )
        .expect("document");
        let root = document.root();
        let node = root.child("theme").and_then(|theme| theme.child("palette"));
        let palette = Palette::from_node(Palette::default(), node);

        assert_eq!(palette.background, Palette::default().background);
        assert_eq!(palette.accent, Color::from_rgb8(0xff, 0x00, 0x00));
    }

    #[test]
    fn opacity_is_decoded_from_config() {
        let document = Document::parse(
            r#"
theme {
    palette {
        background-opacity 0.5
    }
}
"#,
        )
        .expect("document");
        let root = document.root();
        let node = root.child("theme").and_then(|theme| theme.child("palette"));
        let palette = Palette::from_node(Palette::default(), node);

        assert_eq!(palette.background_opacity, 0.5);
    }

    #[test]
    fn opacity_multiplies_background_alpha() {
        let palette = Palette {
            background_opacity: 0.5,
            ..Palette::default()
        };

        assert_eq!(palette.background_color().a, 0.5);
    }
}
