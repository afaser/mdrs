use iced::Element;
use iced::widget::{checkbox, column, container, row, text};

use crate::markdown::item::Bullet;
use crate::markdown::settings::Settings;
use crate::markdown::view::view_with;
use crate::markdown::viewer::Viewer;

pub(crate) fn unordered_list<'a, Message: 'a>(
    viewer: &impl Viewer<'a, Message>,
    settings: Settings,
    bullets: &'a [Bullet],
) -> Element<'a, Message> {
    let rows = bullets.iter().map(|bullet| {
        let marker: Element<'a, Message> = match bullet {
            Bullet::Point { .. } => text("•").size(settings.text_size).into(),
            Bullet::Task { done, .. } => container(checkbox(*done).size(settings.text_size))
                .center_y(
                    iced::advanced::text::LineHeight::default().to_absolute(settings.text_size),
                )
                .into(),
        };

        row([
            marker,
            view_with(
                bullet.items(),
                Settings {
                    spacing: settings.spacing * 0.6,
                    ..settings
                },
                viewer,
            ),
        ])
        .spacing(settings.spacing)
        .into()
    });

    column(rows)
        .spacing(settings.spacing * 0.75)
        .padding([0.0, settings.spacing.0])
        .into()
}

pub(crate) fn ordered_list<'a, Message: 'a>(
    viewer: &impl Viewer<'a, Message>,
    settings: Settings,
    start: u64,
    bullets: &'a [Bullet],
) -> Element<'a, Message> {
    let digits = ((start + bullets.len() as u64).max(1) as f32)
        .log10()
        .ceil();
    let rows = bullets.iter().enumerate().map(|(index, bullet)| {
        row([
            text(format!("{}.", index as u64 + start))
                .size(settings.text_size)
                .align_x(iced::alignment::Horizontal::Right)
                .width(settings.text_size * ((digits / 2.0).ceil() + 1.0))
                .into(),
            view_with(
                bullet.items(),
                Settings {
                    spacing: settings.spacing * 0.6,
                    ..settings
                },
                viewer,
            ),
        ])
        .spacing(settings.spacing)
        .into()
    });

    column(rows).spacing(settings.spacing * 0.75).into()
}
