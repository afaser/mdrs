use iced::widget::markdown;

use crate::theme::palette;

pub fn markdown_style() -> markdown::Style {
    markdown::Style {
        font: crate::theme::regular(),
        inline_code_highlight: markdown::Highlight {
            background: iced::Background::Color(palette::SURFACE),
            border: iced::Border::default(),
        },
        inline_code_padding: iced::Padding {
            top: 2.0,
            right: 4.0,
            bottom: 2.0,
            left: 4.0,
        },
        inline_code_color: palette::TEXT,
        inline_code_font: crate::theme::regular(),
        code_block_font: crate::theme::regular(),
        link_color: palette::GREEN,
    }
}
