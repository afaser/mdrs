use std::path::PathBuf;
use std::time::Instant;

use iced::widget::scrollable::{AbsoluteOffset, Viewport};
use iced::{Subscription, Task};

use crate::app::Content;
use crate::app::Message;
use crate::app::gpu::{self, GpuState};
use crate::app::startup::Startup;
use crate::open::Dialog;
use crate::theme::Viewer;
use crate::watch::FileWatcher;

pub const SCROLL_ID: &str = "document";

pub struct Program {
    content: Content,
    watcher: Option<FileWatcher>,
    viewer: Viewer,
    startup: Option<Startup>,
    gpu: GpuState,
    scroll: AbsoluteOffset,
    dialog: Dialog,
    selection: Option<String>,
    selection_epoch: u64,
}

impl Program {
    pub fn new(started: Instant) -> (Self, Task<Message>) {
        let content = crate::open::path()
            .map(Content::open)
            .unwrap_or(Content::Empty);
        let watcher = content
            .path()
            .map(|path| FileWatcher::new(path.to_path_buf()));
        let startup = Startup::new(started);
        startup.mark("boot");
        let program = Self {
            content,
            watcher,
            viewer: Viewer::new(),
            startup: Some(startup),
            gpu: GpuState::Unknown,
            scroll: AbsoluteOffset::default(),
            dialog: Dialog::default(),
            selection: None,
            selection_epoch: 0,
        };

        (program, Task::none())
    }

    pub fn title(&self) -> String {
        let name = self
            .content
            .path()
            .and_then(std::path::Path::file_name)
            .map(|name| name.to_string_lossy().into_owned());

        match name {
            Some(name) => format!("mdrs — {name}"),
            None => "mdrs".to_owned(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let watcher = match &self.watcher {
            Some(watcher) => watcher.subscription().map(|()| Message::Reload),
            None => Subscription::none(),
        };
        let dialog = self.dialog.subscription().map(Message::Picked);
        let startup = if self.startup.is_some() {
            iced::event::listen_raw(startup_event)
        } else {
            Subscription::none()
        };

        Subscription::batch([
            iced::keyboard::listen().filter_map(keyboard_event),
            iced::event::listen_raw(program_event),
            startup,
            watcher,
            dialog,
        ])
    }

    pub fn content(&self) -> &Content {
        &self.content
    }

    pub fn viewer(&self) -> &Viewer {
        &self.viewer
    }

    pub fn gpu_state(&self) -> &GpuState {
        &self.gpu
    }

    pub fn open(&mut self, path: PathBuf) {
        self.content = Content::open(path.clone());
        self.watcher = Some(FileWatcher::new(path));
        self.scroll = AbsoluteOffset::default();
        self.clear_selections();
    }

    pub fn request_dialog(&mut self) {
        self.dialog.request();
    }

    pub fn finish_dialog(&mut self) {
        self.dialog.finish();
    }

    pub fn reload(&mut self) {
        let previous = std::mem::replace(&mut self.content, Content::Empty);
        self.content = previous.reload();
        self.clear_selections();
    }

    pub fn remember_scroll(&mut self, viewport: Viewport) {
        self.scroll = viewport.absolute_offset();
    }

    pub fn scroll(&self) -> AbsoluteOffset {
        self.scroll
    }

    pub fn selection(&self) -> Option<&str> {
        self.selection.as_deref()
    }

    pub fn selection_epoch(&self) -> u64 {
        self.selection_epoch
    }

    pub fn set_selection(&mut self, selection: Option<String>) {
        self.selection = selection;
    }

    pub fn clear_selections(&mut self) {
        self.selection = None;
        self.selection_epoch = self.selection_epoch.wrapping_add(1);
    }

    pub fn finish_startup(&mut self) {
        if let Some(startup) = self.startup.take() {
            startup.finish();
        }
    }

    pub fn begin_gpu_check(&mut self) -> bool {
        self.gpu.begin_check()
    }

    pub fn apply_gpu(&mut self, adapter: gpu::Gpu) {
        self.gpu.apply(adapter);
    }
}

fn keyboard_event(event: iced::keyboard::Event) -> Option<Message> {
    match event {
        iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Character(character),
            modifiers,
            ..
        } if modifiers.control() && character.as_str().eq_ignore_ascii_case("o") => {
            Some(Message::Choose)
        }
        iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Character(character),
            modifiers,
            ..
        } if modifiers.control() && character.as_str().eq_ignore_ascii_case("c") => {
            Some(Message::CopySelection)
        }
        _ => None,
    }
}

fn startup_event(
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
