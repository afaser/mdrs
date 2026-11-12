use iced::advanced::text::{Alignment, LineHeight};
use iced::border::Radius;
use iced::widget::markdown::Uri;
use iced::widget::scrollable;
use iced::widget::{container, mouse_area, row, scrollable as scrollable_widget, space, svg, text};
use iced::{Background, Border, Color, ContentFit, Element, Font, Length, Pixels, mouse};

use crate::config::{Config, IconSource};
use crate::selection::SelectableText;
use crate::theme::font::FontBook;
use crate::theme::style::scrollbar_style;
use crate::theme::viewer::{CodeCopy, CopyHover};

pub(super) fn content<'a, Message: 'a>(
    config: &Config,
    selectable: SelectableText<'a, Message>,
) -> Element<'a, Message, iced::Theme, iced::Renderer> {
    let bar = &config.code_block.content.scrollbar;
    let padding = config.code_block.content.padding;
    let scroll = scrollable_widget(container(selectable).padding(padding))
        .direction(scrollable::Direction::Horizontal(bar.vertical()))
        .style(scrollbar_style(config, bar))
        .width(Length::Fill);

    container(scroll).width(Length::Fill).into()
}

pub(super) fn header<'a, Message>(
    config: &Config,
    fonts: &FontBook,
    language: Option<&'a str>,
    code: &'a str,
    id: u64,
    copy_progress: Option<f32>,
    hover_progress: f32,
) -> Option<Element<'a, Message, iced::Theme, iced::Renderer>>
where
    Message: From<Uri> + From<Option<String>> + From<CodeCopy> + From<CopyHover> + Clone + 'a,
{
    let header = &config.code_block.header;

    if !header.show {
        return None;
    }

    let palette = &config.palette;
    let label_font = fonts.default_font(&header.label.fonts);
    let logo_font = fonts.default_font(&header.logo.fonts);
    let logo = header.logos.get(language.unwrap_or(&header.label.unknown));
    let logo = icon(
        &logo.icon,
        header.logo.size,
        logo.color.resolve(palette),
        logo_font,
    );
    let label = text(label_text(
        language.unwrap_or(&header.label.unknown),
        header.label.capitalize,
    ))
    .font(label_font)
    .size(header.label.size)
    .color(header.label.color.resolve(palette));
    let hover = header.copy.hover_easing.apply(hover_progress);
    let base_color = header.copy.color.resolve(palette);
    let hover_color = header.copy.hover_color.resolve(palette);
    let (source, icon_color, icon_size) = match copy_progress {
        Some(progress) => {
            let color = header.copy.check_color.resolve(palette);
            let (size, alpha) = check_frame(progress, header.copy.check_easing, header.copy.size);

            (
                &header.copy.check_icon,
                Color {
                    a: color.a * alpha,
                    ..color
                },
                size,
            )
        }
        None => (
            &header.copy.icon,
            blend(base_color, hover_color, hover),
            header.copy.size,
        ),
    };
    let icon = icon(source, icon_size, icon_color, label_font);
    let background = blend(
        header.copy.background.resolve(palette),
        header.copy.hover_background.resolve(palette),
        hover,
    );
    let button_size = header.copy.button_size;
    let radius = header.copy.radius;
    let copy = mouse_area(
        container(icon)
            .width(Length::Fixed(button_size))
            .height(Length::Fixed(button_size))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(background)),
                border: Border {
                    radius: Radius {
                        top_left: radius,
                        top_right: radius,
                        bottom_right: radius,
                        bottom_left: radius,
                    },
                    ..Border::default()
                },
                ..container::Style::default()
            }),
    )
    .interaction(mouse::Interaction::Pointer)
    .on_enter(Message::from(CopyHover { id, hovered: true }))
    .on_exit(Message::from(CopyHover { id, hovered: false }))
    .on_press(Message::from(CodeCopy {
        code: code.to_owned(),
        id,
    }));

    let row = row![logo, label, space::horizontal(), copy]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(header.padding.left);
    let background = header.background.resolve(palette);
    let height = header.height;
    let insets = header.padding.padding();
    let radius =
        (config.code_block.frame.border_radius - config.code_block.frame.border_width).max(0.0);

    Some(
        container(row)
            .height(Length::Fixed(height))
            .padding(insets)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(background)),
                border: Border {
                    radius: Radius {
                        top_left: radius,
                        top_right: radius,
                        ..Radius::default()
                    },
                    ..Border::default()
                },
                ..container::Style::default()
            })
            .into(),
    )
}

fn icon<'a, Message: 'a>(
    source: &IconSource,
    size: f32,
    color: Color,
    font: Font,
) -> Element<'a, Message, iced::Theme, iced::Renderer> {
    match source {
        IconSource::Glyph(glyph) => text(glyph.clone())
            .font(font)
            .size(size)
            .line_height(LineHeight::Absolute(Pixels(size)))
            .color(color)
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .align_x(Alignment::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into(),
        IconSource::EmbeddedSvg(bytes) => svg::Svg::new(svg::Handle::from_memory(*bytes))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .content_fit(ContentFit::Contain)
            .style(move |_theme, _status| svg::Style { color: Some(color) })
            .into(),
        IconSource::Svg(path) => svg::Svg::from_path(path.clone())
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .content_fit(ContentFit::Contain)
            .style(move |_theme, _status| svg::Style { color: Some(color) })
            .into(),
    }
}

fn label_text(language: &str, capitalize: bool) -> String {
    if !capitalize {
        return language.to_owned();
    }

    let mut characters = language.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };

    first.to_uppercase().chain(characters).collect()
}

fn check_frame(progress: f32, easing: crate::config::Easing, size: f32) -> (f32, f32) {
    let growth = (progress / 0.18).clamp(0.0, 1.0);
    let fade = check_fade(progress);
    let scale = 0.7 + 0.3 * easing.apply(growth);

    (size * scale, fade)
}

fn check_fade(progress: f32) -> f32 {
    if progress <= 0.75 {
        return 1.0;
    }

    ((1.0 - progress) / 0.25).clamp(0.0, 1.0)
}

fn blend(from: Color, to: Color, progress: f32) -> Color {
    let progress = progress.clamp(0.0, 1.0);

    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_label_is_capitalized() {
        assert_eq!(label_text("bash", true), "Bash");
        assert_eq!(label_text("bash", false), "bash");
        assert_eq!(label_text("", true), "");
    }

    #[test]
    fn check_frame_pops_in_holds_and_fades() {
        let easing = crate::config::Easing::EaseOut;
        let (start_size, start_alpha) = check_frame(0.0, easing, 20.0);
        let (held_size, held_alpha) = check_frame(0.5, easing, 20.0);
        let (end_size, end_alpha) = check_frame(1.0, easing, 20.0);

        assert_eq!(start_alpha, 1.0);
        assert!(start_size < held_size);
        assert_eq!(held_alpha, 1.0);
        assert!((held_size - 20.0).abs() <= f32::EPSILON);
        assert_eq!(end_alpha, 0.0);
        assert!((end_size - 20.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn blend_interpolates_colors() {
        let from = Color::from_rgba(0.0, 0.0, 0.0, 0.0);
        let to = Color::from_rgba(1.0, 0.5, 0.25, 1.0);
        let middle = blend(from, to, 0.5);

        assert_eq!(middle.r, 0.5);
        assert_eq!(middle.g, 0.25);
        assert_eq!(middle.b, 0.125);
        assert_eq!(middle.a, 0.5);
    }
}
