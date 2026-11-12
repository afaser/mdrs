use std::time::Instant;

use iced::Subscription;

use crate::app::{Message, Program};

pub(super) fn subscription(program: &Program) -> Subscription<Message> {
    let watcher = match program.watcher() {
        Some(watcher) => watcher.subscription().map(|()| Message::Reload),
        None => Subscription::none(),
    };
    let dialog = program.dialog().subscription().map(Message::Picked);
    let redraw = iced::event::listen_raw(redraw_event);
    let scroll = scroll_subscription(program.scrolling());
    let now = Instant::now();
    let animation = animation_subscription(
        program.viewer().copy_animating(now) || program.viewer().hover_animating(now),
    );

    Subscription::batch([
        iced::keyboard::listen().filter_map(keyboard_event),
        iced::event::listen_raw(program_event),
        redraw,
        watcher,
        dialog,
        scroll,
        animation,
    ])
}

fn scroll_subscription(scrolling: bool) -> Subscription<Message> {
    if !scrolling {
        return Subscription::none();
    }

    iced::window::frames().map(Message::ScrollFrame)
}

fn animation_subscription(active: bool) -> Subscription<Message> {
    if !active {
        return Subscription::none();
    }

    iced::window::frames().map(Message::AnimationFrame)
}

fn keyboard_event(event: iced::keyboard::Event) -> Option<Message> {
    let iced::keyboard::Event::KeyPressed {
        key: iced::keyboard::Key::Character(character),
        modifiers,
        ..
    } = event
    else {
        return None;
    };

    if !modifiers.control() {
        return None;
    }

    match character.as_str() {
        value if value.eq_ignore_ascii_case("o") => Some(Message::Choose),
        value if value.eq_ignore_ascii_case("c") => Some(Message::CopySelection),
        _ => None,
    }
}

fn redraw_event(
    event: iced::Event,
    _status: iced::event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        iced::Event::Window(iced::window::Event::RedrawRequested(_)) => Some(Message::Redrawn),
        _ => None,
    }
}

fn program_event(
    event: iced::Event,
    status: iced::event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        iced::Event::Window(iced::window::Event::FileDropped(path)) => Some(Message::Dropped(path)),
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            if status == iced::event::Status::Ignored =>
        {
            Some(Message::ClearSelections)
        }
        _ => None,
    }
}
