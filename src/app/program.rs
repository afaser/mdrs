use std::path::PathBuf;
use std::time::Instant;

use iced::widget::scrollable::{AbsoluteOffset, Viewport};
use iced::{Subscription, Task};

use crate::app::Content;
use crate::app::Message;
use crate::app::Scroll;
use crate::app::gpu::{self, GpuCheck, GpuState};
use crate::app::selection::SelectionState;
use crate::app::startup::Startup;
use crate::app::subscription;
use crate::config::Config;
use crate::open::Dialog;
use crate::theme::{FontBook, Viewer};
use crate::watch::FileWatcher;

pub struct Program {
    config: Config,
    fonts: FontBook,
    content: Content,
    watcher: Option<FileWatcher>,
    viewer: Viewer,
    startup: Option<Startup>,
    gpu: GpuState,
    scroll: Scroll,
    dialog: Dialog,
    selection: SelectionState,
}

impl Program {
    pub(super) fn new(started: Instant, config: Config, fonts: FontBook) -> (Self, Task<Message>) {
        let content = argument_path().map(Content::open).unwrap_or(Content::Empty);
        let watcher = content
            .path()
            .map(|path| FileWatcher::new(path.to_path_buf()));
        let startup = Startup::new(started);

        if let Some(startup) = &startup {
            startup.mark("boot");
        }

        let program = Self {
            config: config.clone(),
            fonts: fonts.clone(),
            content,
            watcher,
            viewer: Viewer::new(config.clone(), fonts),
            startup,
            gpu: GpuState::Unknown,
            scroll: Scroll::new(config.motion.wheel),
            dialog: Dialog::default(),
            selection: SelectionState::default(),
        };

        (program, Task::none())
    }

    pub(super) fn title(&self) -> String {
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

    pub(super) fn subscription(&self) -> Subscription<Message> {
        subscription::subscription(self)
    }

    pub(super) fn config(&self) -> &Config {
        &self.config
    }

    pub(super) fn fonts(&self) -> &FontBook {
        &self.fonts
    }

    pub(super) fn content(&self) -> &Content {
        &self.content
    }

    pub(super) fn viewer(&self) -> &Viewer {
        &self.viewer
    }

    pub(super) fn viewer_mut(&mut self) -> &mut Viewer {
        &mut self.viewer
    }

    pub(super) fn gpu_state(&self) -> &GpuState {
        &self.gpu
    }

    pub(super) fn watcher(&self) -> Option<&FileWatcher> {
        self.watcher.as_ref()
    }

    pub(super) fn dialog(&self) -> &Dialog {
        &self.dialog
    }

    pub(super) fn open(&mut self, path: PathBuf) {
        self.viewer.clear_cache();
        self.content = Content::open(path.clone());
        self.watcher = Some(FileWatcher::new(path));
        self.scroll.reset();
        self.clear_selections();
    }

    pub(super) fn request_dialog(&mut self) {
        self.dialog.request();
    }

    pub(super) fn finish_dialog(&mut self) {
        self.dialog.finish();
    }

    pub(super) fn reload(&mut self) {
        self.viewer.clear_cache();
        let previous = std::mem::take(&mut self.content);
        self.content = previous.reload();
        self.clear_selections();
    }

    pub(super) fn remember_scroll(&mut self, viewport: Viewport) {
        self.scroll.remember(viewport);
    }

    pub(super) fn scroll(&self) -> AbsoluteOffset {
        self.scroll.offset()
    }

    pub(super) fn wheel(&mut self, delta: iced::mouse::ScrollDelta, now: Instant) {
        self.scroll.wheel(delta, now);
    }

    pub(super) fn frame(&mut self, now: Instant) -> Option<AbsoluteOffset> {
        self.scroll.frame(now)
    }

    pub(super) fn scrolling(&self) -> bool {
        self.scroll.moving()
    }

    pub(super) fn stop_scroll(&mut self) {
        self.scroll.stop();
    }

    pub(super) fn selection(&self) -> Option<&str> {
        self.selection.text()
    }

    pub(super) fn set_selection(&mut self, selection: String) {
        self.selection.set(selection);
    }

    pub(super) fn clear_selections(&mut self) {
        self.selection.clear();
    }

    pub(super) fn finish_startup(&mut self) -> Option<f32> {
        self.startup.take().map(Startup::finish)
    }

    pub(super) fn begin_gpu_check(&mut self) -> GpuCheck {
        self.gpu.begin_check()
    }

    pub(super) fn apply_gpu(&mut self, adapter: gpu::Gpu) {
        self.gpu.apply(adapter);
    }

    pub(super) fn sync_viewer_generation(&mut self) {
        self.viewer.set_generation(self.selection.generation());
    }
}

fn argument_path() -> Option<PathBuf> {
    std::env::args_os().nth(1).map(PathBuf::from)
}
