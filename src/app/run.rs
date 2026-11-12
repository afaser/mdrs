use std::time::Instant;

use crate::app::{Program, logging, update, view};
use crate::config::Config;
use crate::theme;

pub fn run() -> iced::Result {
    let started = Instant::now();
    logging::init();

    let config = Config::load();

    if !config.fonts.use_system() {
        theme::isolate_system_fonts();
    }

    let fonts = theme::LoadedFonts::load(&config.fonts);
    let theme = theme::theme(&config);
    let window = window_settings(&config);
    let default_font = fonts.book.default_font(&config.text.body.fonts);
    let state_config = config.clone();
    let state_fonts = fonts.book.clone();

    let mut application = iced::application(
        move || Program::new(started, state_config.clone(), state_fonts.clone()),
        update,
        view,
    )
    .settings(iced::Settings {
        vsync: true,
        ..iced::Settings::default()
    })
    .title(Program::title)
    .theme(theme)
    .default_font(default_font)
    .window(window)
    .centered()
    .subscription(Program::subscription);

    for bytes in fonts.bytes {
        application = application.font(bytes);
    }

    application.run()
}

fn window_settings(config: &Config) -> iced::window::Settings {
    iced::window::Settings {
        size: config.window.size(),
        transparent: config
            .window
            .transparent_for(config.palette.background_color().a),
        ..iced::window::Settings::default()
    }
}
