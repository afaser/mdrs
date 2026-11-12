use iced::Element;

use crate::markdown::{self, HeadingLevel, Settings, Text, Uri};
use crate::selection::BlockSource;
use crate::theme::Viewer;

pub struct DocumentView<'a> {
    viewer: &'a Viewer,
    article: &'a crate::markdown::Article,
}

impl<'a> DocumentView<'a> {
    pub fn new(viewer: &'a Viewer, article: &'a crate::markdown::Article) -> Self {
        Self { viewer, article }
    }

    fn source(&self, text: &Text) -> BlockSource {
        self.viewer
            .source_text(self.article.index(), text)
            .with_copy_index(self.article.copy_index().clone())
    }
}

impl<'a, Message> markdown::Viewer<'a, Message> for DocumentView<'a>
where
    Message: From<Uri>
        + From<Option<String>>
        + From<crate::theme::CodeCopy>
        + From<crate::theme::CopyHover>
        + Clone
        + 'a,
{
    fn image(
        &self,
        settings: Settings,
        url: &'a Uri,
        title: &'a str,
        alt: &'a Text,
    ) -> Element<'a, Message> {
        markdown::Viewer::image(self.viewer, settings, url, title, alt)
    }

    fn heading(
        &self,
        settings: Settings,
        level: &'a HeadingLevel,
        text: &'a Text,
        index: usize,
    ) -> Element<'a, Message> {
        self.viewer
            .heading_content(settings, level, text, index, Some(self.source(text)))
    }

    fn paragraph(&self, settings: Settings, text: &Text) -> Element<'a, Message> {
        self.viewer
            .paragraph_content(settings, text, Some(self.source(text)))
    }

    fn code_block(
        &self,
        settings: Settings,
        language: Option<&'a str>,
        code: &'a str,
        lines: &'a [Text],
    ) -> Element<'a, Message> {
        let source = self
            .viewer
            .source_code(self.article.index(), lines)
            .with_copy_index(self.article.copy_index().clone());

        self.viewer
            .code_block_content(settings, language, code, Some(source))
    }
}
