use iced::Element;
use iced::widget::{scrollable, table as table_widget, text};

use crate::markdown::settings::Settings;
use crate::markdown::view::items;
use crate::markdown::viewer::Viewer;
use crate::markdown::{Alignment, Column, Row};

pub(crate) fn table<'a, Message: 'a>(
    viewer: &impl Viewer<'a, Message>,
    settings: Settings,
    columns: &'a [Column],
    rows: &'a [Row],
) -> Element<'a, Message> {
    let columns = columns.iter().enumerate().map(|(index, column)| {
        table_widget::column(
            items(viewer, settings, &column.header),
            move |row: &Row| match row.cells().get(index) {
                Some(cells) => items(viewer, settings, cells),
                None => text("").into(),
            },
        )
        .align_x(horizontal(column.alignment))
    });
    let table = table_widget(columns, rows)
        .padding_x(settings.spacing.0)
        .padding_y(settings.spacing.0 / 2.0)
        .separator_x(0);

    scrollable(table)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .spacing(settings.spacing.0 / 2.0)
        .into()
}

fn horizontal(alignment: Alignment) -> iced::alignment::Horizontal {
    match alignment {
        Alignment::None | Alignment::Left => iced::alignment::Horizontal::Left,
        Alignment::Center => iced::alignment::Horizontal::Center,
        Alignment::Right => iced::alignment::Horizontal::Right,
    }
}
