use crate::theme::palette;

pub fn catppuccin() -> iced::Theme {
    iced::Theme::custom(
        "Catppuccin Mocha Green",
        iced::theme::Palette {
            background: palette::BACKGROUND,
            text: palette::TEXT,
            primary: palette::GREEN,
            success: palette::GREEN,
            warning: palette::DANGER,
            danger: palette::DANGER,
        },
    )
}
