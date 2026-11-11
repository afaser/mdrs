use std::time::Instant;

use crate::app::{Program, logging, update, view};
use crate::theme;

pub fn run() -> iced::Result {
    let started = Instant::now();
    logging::init();
    theme::isolate_system_fonts();

    iced::application(move || Program::new(started), update, view)
        .title(Program::title)
        .theme(theme::catppuccin())
        .default_font(theme::regular())
        .font(theme::BYTES)
        .window_size(iced::Size::new(1000.0, 750.0))
        .centered()
        .subscription(Program::subscription)
        .run()
}
