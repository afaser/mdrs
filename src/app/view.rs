use iced::widget::{container, markdown, scrollable, text};
use iced::{Element, Length};

use crate::app::program::SCROLL_ID;
use crate::app::{Message, Program, WheelScroll};
use crate::theme;

pub const TEXT_SIZE: f32 = 17.0;
pub const TEXT_LINE_HEIGHT: f32 = 24.0;

pub fn view(program: &Program) -> Element<'_, Message> {
    let body = match program.gpu_state().software_name() {
        Some(name) => message(format!("Программный рендер не поддерживается: {name}")),
        None => content(program),
    };

    container(body)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn content(program: &Program) -> Element<'_, Message> {
    let content = program.content();

    if let Some(path) = content.failed_path() {
        return message(format!("Не удалось открыть файл: {}", path.display()));
    }

    let Some(article) = content.article() else {
        return message("Откройте markdown-файл: Ctrl+O".to_owned());
    };

    program.viewer().set_epoch(program.selection_epoch());

    let document = scrollable(markdown::view_with(
        article.items(),
        settings(),
        program.viewer(),
    ))
    .id(SCROLL_ID)
    .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::default()
            .width(8.0)
            .scroller_width(6.0)
            .spacing(8.0),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .on_scroll(Message::Scrolled);

    WheelScroll::new(document).into()
}

fn settings() -> markdown::Settings {
    let mut settings = markdown::Settings::with_text_size(TEXT_SIZE, theme::markdown_style());
    settings.code_size = iced::Pixels(14.0);

    settings
}

fn message(value: String) -> Element<'static, Message> {
    text(value)
        .font(theme::regular())
        .size(18)
        .color(theme::DANGER)
        .into()
}
