use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::widget::{container, markdown, rich_text, scrollable};
use iced::{Element, Length};

use crate::app::{Message, Program, WheelScroll};
use crate::selection::SelectionSurface;
use crate::theme;
use crate::theme::DocumentView;

pub(super) const SCROLL_ID: &str = "document";

pub fn view(program: &Program) -> Element<'_, Message> {
    let body = match program.gpu_state().software_name() {
        Some(name) => message(
            program,
            format!("Программный рендер не поддерживается: {name}"),
        ),
        None => content(program),
    };

    let document = container(body)
        .padding(program.config().window.padding)
        .width(Length::Fill)
        .height(Length::Fill);

    SelectionSurface::new(
        document,
        program.document_selection(),
        program.geometry(),
        program.scroll_offset(),
        Message::from,
    )
    .into()
}

fn content(program: &Program) -> Element<'_, Message> {
    let content = program.content();

    if let Some(path) = content.failed_path() {
        return message(
            program,
            format!("Не удалось открыть файл: {}", path.display()),
        );
    }

    let Some(article) = content.article() else {
        return message(program, "Откройте markdown-файл: Ctrl+O".to_owned());
    };

    let config = program.config();
    let bar = &config.scrollbars.document;
    let viewer = DocumentView::new(program.viewer(), article);
    let document = scrollable(markdown::view_with(
        article.items(),
        theme::markdown_settings(config, program.fonts()),
        &viewer,
    ))
    .id(SCROLL_ID)
    .direction(scrollable::Direction::Vertical(bar.vertical()))
    .style(theme::scrollbar_style(config, bar))
    .width(Length::Fill)
    .height(Length::Fill);

    WheelScroll::new(
        document,
        config.motion.wheel,
        program.generation(),
        program.scroll_offset(),
    )
    .into()
}

fn message(program: &Program, value: String) -> Element<'static, Message> {
    let config = program.config();

    let font = program
        .fonts()
        .role_font(
            &config.text.message,
            config.text.message.weight,
            iced::font::Style::Normal,
        )
        .unwrap_or_else(|| program.fonts().default_font(&config.text.message.fonts));
    let letter_spacing = (config.text.message.letter_spacing > 0.0)
        .then_some(iced::Pixels(config.text.message.letter_spacing));
    let span: Span<'static, Uri> = Span::new(value)
        .font(font)
        .size(config.text.message.size)
        .color(config.text.message.color.resolve(&config.palette));
    let span = match letter_spacing {
        Some(letter_spacing) => span.letter_spacing(letter_spacing),
        None => span,
    };

    rich_text(vec![span]).into()
}
