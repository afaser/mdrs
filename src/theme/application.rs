use iced::Theme;

use crate::config::Config;

pub fn theme(config: &Config) -> Theme {
    let palette = config.palette;

    Theme::custom(
        "mdrs",
        iced::theme::Palette {
            background: palette.background_color(),
            text: palette.text,
            primary: palette.primary,
            success: palette.success,
            warning: palette.warning,
            danger: palette.danger,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_uses_configured_palette() {
        let config = Config::default();
        let theme = theme(&config);

        assert_eq!(theme.palette().background, config.palette.background);
        assert_eq!(theme.palette().primary, config.palette.accent);
    }

    #[test]
    fn accent_override_changes_theme() {
        let mut config = Config::default();
        config.palette.accent = iced::Color::from_rgb8(0xff, 0x00, 0x00);
        config.palette.primary = config.palette.accent;

        assert_eq!(
            theme(&config).palette().primary,
            iced::Color::from_rgb8(0xff, 0x00, 0x00)
        );
    }
}
