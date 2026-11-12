use iced::Element;
use iced::widget::{column, row, rule};

use crate::markdown::item::Item;
use crate::markdown::settings::Settings;
use crate::markdown::view::item;
use crate::markdown::viewer::Viewer;

pub(crate) fn quote<'a, Message: 'a>(
    viewer: &impl Viewer<'a, Message>,
    settings: Settings,
    contents: &'a [Item],
) -> Element<'a, Message> {
    row([
        rule::vertical(4).into(),
        column(
            contents
                .iter()
                .enumerate()
                .map(|(index, content)| item(viewer, settings, content, index)),
        )
        .spacing(settings.spacing.0)
        .into(),
    ])
    .height(iced::Length::Shrink)
    .spacing(settings.spacing.0)
    .into()
}
