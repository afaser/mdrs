use iced::widget::container;
use iced::widget::markdown;
use iced::widget::scrollable as scrollable_widget;
use iced::widget::scrollable::{self, Rail, Scroller, Status};
use iced::{Background, Border, Color, Pixels, Shadow};

use crate::config::Bar;
use crate::config::Config;
use crate::theme::font::FontBook;

pub fn markdown_settings(config: &Config, book: &FontBook) -> markdown::Settings {
    let text = &config.text;

    markdown::Settings {
        text_size: Pixels(text.body.size),
        h1_size: Pixels(text.heading.h1.size),
        h2_size: Pixels(text.heading.h2.size),
        h3_size: Pixels(text.heading.h3.size),
        h4_size: Pixels(text.heading.h4.size),
        h5_size: Pixels(text.heading.h5.size),
        h6_size: Pixels(text.heading.h6.size),
        code_size: Pixels(config.code_block.text.size),
        spacing: Pixels(text.spacing),
        style: markdown_style(config, book),
    }
}

fn markdown_style(config: &Config, book: &FontBook) -> markdown::Style {
    let palette = &config.palette;
    let text = &config.text;
    let font = book
        .role_font(&text.body, text.body.weight, iced::font::Style::Normal)
        .unwrap_or_else(|| book.default_font(&text.body.fonts));
    let inline_font = book
        .role_font(
            &text.inline_code.role,
            text.inline_code.role.weight,
            iced::font::Style::Normal,
        )
        .unwrap_or_else(|| book.default_font(&text.inline_code.role.fonts));
    let code_font = book
        .role_font(
            &config.code_block.text,
            config.code_block.text.weight,
            iced::font::Style::Normal,
        )
        .unwrap_or_else(|| book.default_font(&config.code_block.text.fonts));

    markdown::Style {
        font,
        inline_code_highlight: markdown::Highlight {
            background: Background::Color(text.inline_code.background.resolve(palette)),
            border: iced::border::rounded(text.inline_code.radius),
        },
        inline_code_padding: text.inline_code.padding.padding(),
        inline_code_color: text.inline_code.role.color.resolve(palette),
        inline_code_font: inline_font,
        code_block_font: code_font,
        link_color: text.link.color.resolve(palette),
    }
}

pub fn heading_size(settings: markdown::Settings, level: &markdown::HeadingLevel) -> Pixels {
    match level {
        markdown::HeadingLevel::H1 => settings.h1_size,
        markdown::HeadingLevel::H2 => settings.h2_size,
        markdown::HeadingLevel::H3 => settings.h3_size,
        markdown::HeadingLevel::H4 => settings.h4_size,
        markdown::HeadingLevel::H5 => settings.h5_size,
        markdown::HeadingLevel::H6 => settings.h6_size,
    }
}

pub fn heading_top(config: &Config, index: usize) -> Pixels {
    if index == 0 {
        return Pixels::ZERO;
    }

    Pixels(config.text.heading_gap)
}

pub fn scrollbar_style(
    config: &Config,
    bar: &Bar,
) -> impl Fn(&iced::Theme, Status) -> scrollable::Style + 'static {
    let palette = config.palette;
    let track = bar.track.resolve(&palette);
    let thumb = bar.thumb.resolve(&palette);
    let hover = bar.thumb_hover.resolve(&palette);

    move |_theme, status| {
        let hovered = matches!(status, Status::Hovered { .. } | Status::Dragged { .. });
        let thumb = thumb_color(hovered, thumb, hover);

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail(track, thumb),
            horizontal_rail: rail(track, thumb),
            gap: None,
            auto_scroll: auto_scroll(),
        }
    }
}

fn thumb_color(active: bool, idle: Color, hovered: Color) -> Color {
    if active {
        return hovered;
    }

    idle
}

fn rail(track: Color, thumb: Color) -> Rail {
    Rail {
        background: Some(Background::Color(track)),
        border: Border::default(),
        scroller: Scroller {
            background: Background::Color(thumb),
            border: Border::default(),
        },
    }
}

fn auto_scroll() -> scrollable_widget::AutoScroll {
    scrollable_widget::AutoScroll {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        shadow: Shadow::default(),
        icon: Color::TRANSPARENT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{FontBook, isolate_system_fonts};

    #[test]
    fn inline_code_radius_reaches_markdown_style() {
        isolate_system_fonts();
        let mut config = Config::default();
        config.text.inline_code.radius = 7.0;
        let book = FontBook::new(&config.fonts);
        let style = markdown_style(&config, &book);

        assert_eq!(style.inline_code_highlight.border.radius.top_left, 7.0);
        assert_eq!(style.inline_code_highlight.border.radius.bottom_right, 7.0);
    }
}
