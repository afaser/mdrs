use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;

use iced::widget::{container, image as image_widget};
use iced::{ContentFit, Element, Length, padding};

use crate::config::Config;
use crate::markdown::SourceIndex;
use crate::markdown::{HeadingLevel, Settings, Text, Uri, Viewer as MarkdownViewer};
use crate::selection::BlockSource;
use crate::selection::DocumentGeometry;
use crate::selection::DocumentSelection;
use crate::text::Label;
use crate::theme::font::FontBook;
use crate::theme::highlighter::Highlighter;
use crate::theme::style::heading_top;
use crate::theme::viewer_text::{TextRole, ViewerText};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeCopy {
    pub code: String,
    pub id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyHover {
    pub id: u64,
    pub hovered: bool,
}

struct CopyFeedback {
    id: u64,
    started: Instant,
}

struct HoverFeedback {
    id: u64,
    from: f32,
    to: f32,
    value: f32,
    started: Instant,
    animating: bool,
}

pub struct Viewer {
    highlighter: Highlighter,
    text: ViewerText,
    copy: RefCell<Option<CopyFeedback>>,
    hover: RefCell<Option<HoverFeedback>>,
    images: RefCell<HashMap<Uri, iced::widget::image::Handle>>,
    sources: RefCell<HashMap<usize, BlockSource>>,
}

impl Viewer {
    pub fn new(
        config: Config,
        fonts: FontBook,
        selection: std::rc::Rc<std::cell::RefCell<DocumentSelection>>,
        geometry: std::rc::Rc<std::cell::RefCell<DocumentGeometry>>,
    ) -> Self {
        let text = ViewerText::new(config, fonts, selection, geometry);
        let highlighter = Highlighter::new(&text.config().highlighting, &text.config().palette);

        Self {
            highlighter,
            text,
            copy: RefCell::new(None),
            hover: RefCell::new(None),
            images: RefCell::new(HashMap::new()),
            sources: RefCell::new(HashMap::new()),
        }
    }

    pub fn set_generation(&mut self, generation: crate::selection::Generation) {
        self.text.set_generation(generation);
    }

    pub fn clear_cache(&self) {
        self.text.clear();
        *self.copy.borrow_mut() = None;
        *self.hover.borrow_mut() = None;
        self.images.borrow_mut().clear();
        self.sources.borrow_mut().clear();
    }

    pub fn copied(&self, id: u64, now: Instant) {
        *self.copy.borrow_mut() = Some(CopyFeedback { id, started: now });
    }

    pub fn copy_progress(&self, id: u64, now: Instant) -> Option<f32> {
        let mut copy = self.copy.borrow_mut();
        let feedback = copy.as_mut()?;

        if feedback.id != id {
            return None;
        }

        let duration = self.copy_duration();
        let elapsed = now
            .saturating_duration_since(feedback.started)
            .as_secs_f32();

        if duration <= f32::EPSILON || elapsed >= duration {
            *copy = None;
            return None;
        }

        Some((elapsed / duration).clamp(0.0, 1.0))
    }

    pub fn hovered(&self, id: u64, hovered: bool, now: Instant) {
        if !hovered
            && self
                .hover
                .borrow()
                .as_ref()
                .is_some_and(|feedback| feedback.id != id)
        {
            return;
        }

        let current = self.hover_progress(id, now).unwrap_or(0.0);
        let target = match hovered {
            true => 1.0,
            false => 0.0,
        };
        let duration = self.hover_duration();
        let mut hover = self.hover.borrow_mut();

        if duration <= f32::EPSILON {
            *hover = Some(HoverFeedback {
                id,
                from: target,
                to: target,
                value: target,
                started: now,
                animating: false,
            });
            return;
        }

        *hover = Some(HoverFeedback {
            id,
            from: current,
            to: target,
            value: current,
            started: now,
            animating: true,
        });
    }

    pub fn hover_progress(&self, id: u64, now: Instant) -> Option<f32> {
        let mut hover = self.hover.borrow_mut();
        let feedback = hover.as_mut()?;

        if feedback.id != id {
            return None;
        }

        if !feedback.animating {
            return Some(feedback.value);
        }

        let duration = self.hover_duration();
        let elapsed = now
            .saturating_duration_since(feedback.started)
            .as_secs_f32();
        let progress = (elapsed / duration).clamp(0.0, 1.0);
        let value = feedback.from + (feedback.to - feedback.from) * progress;

        if elapsed >= duration {
            feedback.value = feedback.to;
            feedback.animating = false;
            return Some(feedback.to);
        }

        feedback.value = value;
        Some(value)
    }

    pub fn hover_animating(&self, now: Instant) -> bool {
        let hover = self.hover.borrow();
        let Some(feedback) = hover.as_ref() else {
            return false;
        };

        feedback.animating
            && now
                .saturating_duration_since(feedback.started)
                .as_secs_f32()
                < self.hover_duration()
    }

    pub fn tick_hover(&self, now: Instant) {
        let mut hover = self.hover.borrow_mut();
        let Some(feedback) = hover.as_mut() else {
            return;
        };

        if !feedback.animating {
            return;
        }

        if now
            .saturating_duration_since(feedback.started)
            .as_secs_f32()
            >= self.hover_duration()
        {
            feedback.value = feedback.to;
            feedback.animating = false;
        }
    }

    pub fn copy_animating(&self, now: Instant) -> bool {
        let copy = self.copy.borrow();
        let Some(feedback) = copy.as_ref() else {
            return false;
        };

        now.saturating_duration_since(feedback.started)
            .as_secs_f32()
            < self.copy_duration()
    }

    pub fn tick_copy(&self, now: Instant) {
        let expired = {
            let copy = self.copy.borrow();
            copy.as_ref().is_some_and(|feedback| {
                now.saturating_duration_since(feedback.started)
                    .as_secs_f32()
                    >= self.copy_duration()
            })
        };

        if expired {
            *self.copy.borrow_mut() = None;
        }
    }

    pub(super) fn source_text(&self, index: &Arc<SourceIndex>, text: &Text) -> BlockSource {
        let key = text.source().start;

        if let Some(source) = self.sources.borrow().get(&key) {
            return source.clone();
        }

        let source = BlockSource::from_text(key, Arc::clone(index), text);
        self.sources.borrow_mut().insert(key, source.clone());
        source
    }

    pub(super) fn source_code(&self, index: &Arc<SourceIndex>, lines: &[Text]) -> BlockSource {
        let key = lines
            .first()
            .map(|line| line.source().start)
            .unwrap_or_default();

        if let Some(source) = self.sources.borrow().get(&key) {
            return source.clone();
        }

        let source = BlockSource::from_code(key, Arc::clone(index), lines);
        self.sources.borrow_mut().insert(key, source.clone());
        source
    }

    fn copy_duration(&self) -> f32 {
        self.text.config().code_block.header.copy.check_duration
    }

    fn hover_duration(&self) -> f32 {
        self.text.config().code_block.header.copy.hover_duration
    }
}

impl<'a, Message> MarkdownViewer<'a, Message> for Viewer
where
    Message: From<Uri> + From<Option<String>> + From<CodeCopy> + From<CopyHover> + Clone + 'a,
{
    fn heading(
        &self,
        settings: Settings,
        level: &'a HeadingLevel,
        text: &'a Text,
        index: usize,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        self.heading_content(settings, level, text, index, None)
    }

    fn paragraph(
        &self,
        settings: Settings,
        text: &Text,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        self.paragraph_content(settings, text, None)
    }

    fn image(
        &self,
        settings: Settings,
        url: &'a Uri,
        _title: &'a str,
        alt: &'a Text,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        match crate::markdown::file_path(url).filter(|path| path.is_file()) {
            Some(path) => {
                let handle = self
                    .images
                    .borrow_mut()
                    .entry(url.clone())
                    .or_insert_with(|| iced::widget::image::Handle::from_path(path))
                    .clone();

                image_widget(handle)
                    .content_fit(ContentFit::Contain)
                    .width(Length::Fill)
                    .into()
            }
            None => container(Label::new(alt.spans(settings.style)).size(settings.text_size))
                .padding(settings.spacing)
                .into(),
        }
    }

    fn code_block(
        &self,
        settings: Settings,
        language: Option<&'a str>,
        code: &'a str,
        lines: &'a [Text],
    ) -> Element<'a, Message, iced::Theme, iced::Renderer> {
        let _ = lines;
        self.code_block_content(settings, language, code, None)
    }
}

impl Viewer {
    pub(super) fn heading_content<'a, Message>(
        &self,
        settings: Settings,
        level: &'a HeadingLevel,
        text: &'a Text,
        index: usize,
        source: Option<BlockSource>,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer>
    where
        Message: From<Uri> + From<Option<String>> + 'a,
    {
        let size = settings.heading_size(level);
        let selectable = self.text.flow_text(
            TextRole::Heading(heading_index(level)),
            text.spans(settings.style),
            size,
            source,
        );

        container(selectable)
            .padding(padding::top(heading_top(self.text.config(), index)))
            .into()
    }

    pub(super) fn paragraph_content<'a, Message>(
        &self,
        settings: Settings,
        text: &Text,
        source: Option<BlockSource>,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer>
    where
        Message: From<Uri> + From<Option<String>> + 'a,
    {
        self.text.flow_text(
            TextRole::Paragraph,
            text.spans(settings.style),
            settings.text_size,
            source,
        )
    }

    pub(super) fn code_block_content<'a, Message>(
        &self,
        settings: Settings,
        language: Option<&'a str>,
        code: &'a str,
        source: Option<BlockSource>,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer>
    where
        Message: From<Uri> + From<Option<String>> + From<CodeCopy> + From<CopyHover> + Clone + 'a,
    {
        let spans = self.highlighter.block(language.unwrap_or_default(), code);
        let selectable =
            self.text
                .selectable_text(TextRole::Code, spans, settings.code_size, source);
        let config = self.text.config();
        let content = crate::theme::code_block::content(config, selectable);
        let id = block_id(language.unwrap_or_default(), code);
        let now = Instant::now();
        let progress = self.copy_progress(id, now);
        let hover = self.hover_progress(id, now).unwrap_or(0.0);
        let body: Element<'a, Message, iced::Theme, iced::Renderer> =
            match crate::theme::code_block::header(
                config,
                self.text.fonts(),
                language,
                code,
                id,
                progress,
                hover,
            ) {
                Some(header) => iced::widget::column![header, content].into(),
                None => content,
            };

        let frame = &config.code_block.frame;
        let inner = container(body)
            .padding(frame.border_width)
            .width(Length::Fill);
        let border = iced::border::rounded(frame.border_radius)
            .color(frame.border.resolve(&config.palette))
            .width(frame.border_width);

        container(crate::theme::frame::frame(
            inner,
            frame.background.resolve(&config.palette),
            border,
        ))
        .padding(frame.margin)
        .into()
    }
}

fn block_id(language: &str, code: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    language.hash(&mut hasher);
    code.hash(&mut hasher);

    hasher.finish()
}

fn heading_index(level: &HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 0,
        HeadingLevel::H2 => 1,
        HeadingLevel::H3 => 2,
        HeadingLevel::H4 => 3,
        HeadingLevel::H5 => 4,
        HeadingLevel::H6 => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn copy_progress_starts_and_expires() {
        let config = Config::default();
        let viewer = Viewer::new(
            config.clone(),
            FontBook::new(&config.fonts),
            Default::default(),
            Default::default(),
        );
        let start = Instant::now();

        viewer.copied(7, start);
        assert_eq!(viewer.copy_progress(7, start), Some(0.0));
        assert!(viewer.copy_animating(start + Duration::from_millis(600)));

        let middle = viewer
            .copy_progress(7, start + Duration::from_millis(600))
            .expect("progress");
        assert!(middle > 0.0 && middle < 1.0);
        assert_eq!(
            viewer.copy_progress(7, start + Duration::from_millis(1300)),
            None
        );
    }

    #[test]
    fn hover_progress_fades_in_and_holds() {
        let config = Config::default();
        let viewer = Viewer::new(
            config.clone(),
            FontBook::new(&config.fonts),
            Default::default(),
            Default::default(),
        );
        let start = Instant::now();

        viewer.hovered(7, true, start);
        assert_eq!(viewer.hover_progress(7, start), Some(0.0));
        assert!(viewer.hover_animating(start + Duration::from_millis(75)));

        let middle = viewer
            .hover_progress(7, start + Duration::from_millis(75))
            .expect("progress");
        assert!(middle > 0.0 && middle < 1.0);
        assert_eq!(
            viewer.hover_progress(7, start + Duration::from_millis(300)),
            Some(1.0)
        );
        assert!(!viewer.hover_animating(start + Duration::from_millis(200)));
    }

    #[test]
    fn stale_exit_does_not_clear_new_hover() {
        let config = Config::default();
        let viewer = Viewer::new(
            config.clone(),
            FontBook::new(&config.fonts),
            Default::default(),
            Default::default(),
        );
        let start = Instant::now();

        viewer.hovered(1, true, start);
        viewer.hovered(2, true, start + Duration::from_millis(10));
        viewer.hovered(1, false, start + Duration::from_millis(20));

        assert_eq!(
            viewer.hover_progress(2, start + Duration::from_millis(300)),
            Some(1.0)
        );
    }
}
