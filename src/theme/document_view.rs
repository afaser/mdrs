use iced::Element;
use iced::widget::markdown::{self, Bullet, Column, HeadingLevel, Item, Row, Settings, Text, Uri};

use crate::markdown::Article;
use crate::selection::BlockSource;
use crate::theme::Viewer;

pub struct DocumentView<'a> {
    viewer: &'a Viewer,
    article: &'a Article,
}

impl<'a> DocumentView<'a> {
    pub fn new(viewer: &'a Viewer, article: &'a Article) -> Self {
        Self { viewer, article }
    }

    fn source(&self, text: &Text) -> BlockSource {
        self.viewer
            .source_text(self.article.index(), text)
            .with_copy_index(self.article.copy_index().clone())
    }
}

impl<'a, Message> markdown::Viewer<'a, Message, iced::Theme, iced::Renderer> for DocumentView<'a>
where
    Message: From<Uri>
        + From<Option<String>>
        + From<crate::theme::CodeCopy>
        + From<crate::theme::CopyHover>
        + Clone
        + 'a,
{
    fn on_link_click(url: Uri) -> Message {
        Message::from(url)
    }

    fn image(
        &self,
        settings: Settings,
        url: &'a Uri,
        title: &'a str,
        alt: &Text,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        markdown::Viewer::image(self.viewer, settings, url, title, alt)
    }

    fn heading(
        &self,
        settings: Settings,
        level: &'a HeadingLevel,
        text: &'a Text,
        index: usize,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        self.viewer
            .heading_content(settings, level, text, index, Some(self.source(text)))
    }

    fn paragraph(
        &self,
        settings: Settings,
        text: &Text,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        self.viewer
            .paragraph_content(settings, text, Some(self.source(text)))
    }

    fn code_block(
        &self,
        settings: Settings,
        language: Option<&'a str>,
        code: &'a str,
        lines: &'a [Text],
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        let source = self
            .viewer
            .source_code(self.article.index(), lines)
            .with_copy_index(self.article.copy_index().clone());

        self.viewer
            .code_block_content(settings, language, code, Some(source))
    }

    fn unordered_list(
        &self,
        settings: Settings,
        bullets: &'a [Bullet],
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        markdown::unordered_list(self, settings, bullets)
    }

    fn ordered_list(
        &self,
        settings: Settings,
        start: u64,
        bullets: &'a [Bullet],
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        markdown::ordered_list(self, settings, start, bullets)
    }

    fn quote(
        &self,
        settings: Settings,
        contents: &'a [Item],
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        markdown::quote(self, settings, contents)
    }

    fn rule(&self, _settings: Settings) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        markdown::rule()
    }

    fn table(
        &self,
        settings: Settings,
        columns: &'a [Column],
        rows: &'a [Row],
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        markdown::table(self, settings, columns, rows)
    }
}
