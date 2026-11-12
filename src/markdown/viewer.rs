use iced::Element;
use iced::widget::{column, container};

use crate::markdown::settings::Settings;
use crate::markdown::text::Text;
use crate::markdown::uri::Uri;
use crate::markdown::{Bullet, Column, HeadingLevel, Item, Row};
use crate::markdown::{list_view, quote_view, table_view};
use crate::text::Label;

pub trait Viewer<'a, Message: 'a>: Sized {
    fn image(
        &self,
        settings: Settings,
        url: &'a Uri,
        title: &'a str,
        alt: &'a Text,
    ) -> Element<'a, Message> {
        let _url = url;
        let _title = title;

        container(Label::new(alt.spans(settings.style)).size(settings.text_size))
            .padding(settings.spacing)
            .into()
    }

    fn heading(
        &self,
        settings: Settings,
        level: &'a HeadingLevel,
        text: &'a Text,
        index: usize,
    ) -> Element<'a, Message> {
        heading(settings, level, text, index)
    }

    fn paragraph(&self, settings: Settings, text: &Text) -> Element<'a, Message> {
        paragraph(settings, text)
    }

    fn code_block(
        &self,
        settings: Settings,
        language: Option<&'a str>,
        code: &'a str,
        lines: &'a [Text],
    ) -> Element<'a, Message> {
        let _language = language;
        let _code = code;

        code_block(settings, lines)
    }

    fn unordered_list(&self, settings: Settings, bullets: &'a [Bullet]) -> Element<'a, Message> {
        list_view::unordered_list(self, settings, bullets)
    }

    fn ordered_list(
        &self,
        settings: Settings,
        start: u64,
        bullets: &'a [Bullet],
    ) -> Element<'a, Message> {
        list_view::ordered_list(self, settings, start, bullets)
    }

    fn quote(&self, settings: Settings, contents: &'a [Item]) -> Element<'a, Message> {
        quote_view::quote(self, settings, contents)
    }

    fn rule(&self, _settings: Settings) -> Element<'a, Message> {
        iced::widget::rule::horizontal(2).into()
    }

    fn table(
        &self,
        settings: Settings,
        columns: &'a [Column],
        rows: &'a [Row],
    ) -> Element<'a, Message> {
        table_view::table(self, settings, columns, rows)
    }
}

fn heading<'a, Message: 'a>(
    settings: Settings,
    level: &HeadingLevel,
    text: &Text,
    index: usize,
) -> Element<'a, Message> {
    let size = settings.heading_size(level);
    let top = match index {
        0 => iced::Pixels::ZERO,
        _ => settings.text_size / 2.0,
    };

    container(Label::new(text.spans(settings.style)).size(size))
        .padding(iced::padding::top(top))
        .into()
}

fn paragraph<'a, Message: 'a>(settings: Settings, text: &Text) -> Element<'a, Message> {
    Label::new(text.spans(settings.style))
        .size(settings.text_size)
        .into()
}

fn code_block<'a, Message: 'a>(settings: Settings, lines: &[Text]) -> Element<'a, Message> {
    let code = column(lines.iter().map(|line| {
        Label::new(line.spans(settings.style))
            .font(settings.style.code_block_font)
            .size(settings.code_size)
            .into()
    }));

    container(
        iced::widget::scrollable(container(code).padding(settings.code_size)).direction(
            iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::default()
                    .width(settings.code_size / 2.0)
                    .scroller_width(settings.code_size / 2.0),
            ),
        ),
    )
    .width(iced::Length::Fill)
    .padding(settings.code_size / 4.0)
    .style(container::dark)
    .into()
}
