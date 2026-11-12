mod app;
mod config;
mod document;
mod markdown;
mod open;
mod selection;
mod theme;
mod watch;

pub fn run() -> iced::Result {
    app::run()
}
