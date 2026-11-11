use std::cell::Cell;

use iced::advanced::text::Wrapping;
use iced::widget::markdown::{self, Settings, Text, Uri};
use iced::widget::{container, image as image_widget, rich_text, scrollable};
use iced::{Background, ContentFit, Element, Length, Pixels, border, padding};

use crate::selection::SelectableText;
use crate::theme::highlight::Highlighter;
use crate::theme::palette;

#[derive(Default)]
pub struct Viewer {
    highlighter: Highlighter,
    epoch: Cell<u64>,
}

impl Viewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_epoch(&self, epoch: u64) {
        self.epoch.set(epoch);
    }
}

impl<'a, Message> markdown::Viewer<'a, Message, iced::Theme, iced::Renderer> for Viewer
where
    Message: From<Uri> + From<Option<String>> + 'a,
{
    fn on_link_click(url: Uri) -> Message {
        Message::from(url)
    }

    fn heading(
        &self,
        settings: Settings,
        level: &'a markdown::HeadingLevel,
        text: &'a Text,
        index: usize,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        let size = heading_size(settings, level);
        let top = if index > 0 {
            settings.text_size / 2.0
        } else {
            Pixels::ZERO
        };

        container(
            SelectableText::new(text.spans(settings.style), size, settings.style.font)
                .color(palette::GREEN)
                .selection_color(palette::SELECTION)
                .height(Length::Shrink)
                .epoch(self.epoch.get())
                .on_link_click(Message::from)
                .on_selection(Message::from),
        )
        .padding(padding::top(top))
        .into()
    }

    fn paragraph(
        &self,
        settings: Settings,
        text: &Text,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        SelectableText::new(
            text.spans(settings.style),
            settings.text_size,
            settings.style.font,
        )
        .color(palette::TEXT)
        .selection_color(palette::SELECTION)
        .height(Length::Shrink)
        .epoch(self.epoch.get())
        .on_link_click(Message::from)
        .on_selection(Message::from)
        .into()
    }

    fn image(
        &self,
        settings: Settings,
        url: &'a Uri,
        _title: &'a str,
        alt: &Text,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        match crate::markdown::file_path(url).filter(|path| path.is_file()) {
            Some(path) => image_widget(iced::widget::image::Handle::from_path(path))
                .content_fit(ContentFit::Contain)
                .width(Length::Fill)
                .into(),
            None => container(rich_text(alt.spans(settings.style)).size(settings.text_size))
                .padding(settings.spacing)
                .into(),
        }
    }

    fn code_block(
        &self,
        settings: Settings,
        language: Option<&'a str>,
        code: &'a str,
        _lines: &'a [Text],
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        let spans = self.highlighter.block(language.unwrap_or_default(), code);
        let text = SelectableText::new(spans, settings.code_size, settings.style.code_block_font)
            .color(palette::TEXT)
            .selection_color(palette::SELECTION)
            .wrapping(Wrapping::None)
            .epoch(self.epoch.get())
            .on_selection(Message::from);

        container(
            scrollable(container(text).padding(settings.code_size)).direction(
                scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default()
                        .width(settings.code_size / 2.0)
                        .scroller_width(settings.code_size / 2.0)
                        .spacing(settings.code_size / 4.0),
                ),
            ),
        )
        .width(Length::Fill)
        .padding(settings.code_size / 4.0)
        .style(code_style)
        .into()
    }
}

fn heading_size(settings: Settings, level: &markdown::HeadingLevel) -> Pixels {
    match level {
        markdown::HeadingLevel::H1 => settings.h1_size,
        markdown::HeadingLevel::H2 => settings.h2_size,
        markdown::HeadingLevel::H3 => settings.h3_size,
        markdown::HeadingLevel::H4 => settings.h4_size,
        markdown::HeadingLevel::H5 => settings.h5_size,
        markdown::HeadingLevel::H6 => settings.h6_size,
    }
}

fn code_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette::SURFACE)),
        border: border::rounded(6.0).color(palette::BORDER).width(1.0),
        ..container::Style::default()
    }
}
