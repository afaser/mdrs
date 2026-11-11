use iced::Task;
use iced::widget::scrollable::AbsoluteOffset;

use crate::app::gpu;
use crate::app::program::SCROLL_ID;
use crate::app::{Message, Program};
use crate::open;

pub fn update(program: &mut Program, message: Message) -> Task<Message> {
    match message {
        Message::Choose => {
            program.request_dialog();
            Task::none()
        }
        Message::ClearSelections => {
            program.clear_selections();
            Task::none()
        }
        Message::CopySelection => match program.selection() {
            Some(text) => iced::clipboard::write(text.to_owned()),
            None => Task::none(),
        },
        Message::Dropped(path) => open_document(program, path),
        Message::Gpu(adapter) => {
            program.apply_gpu(adapter);
            Task::none()
        }
        Message::Link(url) => {
            open::launch(&url);
            Task::none()
        }
        Message::Picked(Some(path)) => {
            program.finish_dialog();
            open_document(program, path)
        }
        Message::Picked(None) => {
            program.finish_dialog();
            Task::none()
        }
        Message::Redrawn => first_frame(program),
        Message::Reload => reload(program),
        Message::Scrolled(viewport) => {
            program.remember_scroll(viewport);
            Task::none()
        }
        Message::ScrollFrame(now) => match program.frame(now) {
            Some(offset) => scroll_to(offset),
            None => Task::none(),
        },
        Message::Selected(Some(selection)) => {
            program.set_selection(Some(selection));
            Task::none()
        }
        Message::Selected(None) => {
            program.clear_selections();
            Task::none()
        }
        Message::Wheeled(delta) => {
            program.wheel(delta);
            Task::none()
        }
    }
}

fn open_document(program: &mut Program, path: std::path::PathBuf) -> Task<Message> {
    program.stop_scroll();
    program.open(path);
    scroll_to(AbsoluteOffset::default())
}

fn reload(program: &mut Program) -> Task<Message> {
    program.stop_scroll();
    program.reload();
    let offset = program.scroll();

    scroll_to(offset)
}

fn scroll_to(offset: AbsoluteOffset) -> Task<Message> {
    iced_runtime::widget::operation::scroll_to(SCROLL_ID, offset)
}

fn first_frame(program: &mut Program) -> Task<Message> {
    program.finish_startup();

    if program.begin_gpu_check() {
        return Task::perform(async { gpu::detect() }, Message::Gpu);
    }

    Task::none()
}
