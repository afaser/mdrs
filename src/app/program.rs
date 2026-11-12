use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use iced::widget::scrollable::AbsoluteOffset;
use iced::{Subscription, Task};

use crate::app::Content;
use crate::app::Message;
use crate::app::gpu::{self, GpuCheck, GpuState};
use crate::app::selection::SelectionState;
use crate::app::startup::Startup;
use crate::app::subscription;
use crate::config::Config;
use crate::open::Dialog;
use crate::selection::{DocumentGeometry, DocumentSelection};
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
    dialog: Dialog,
    selection: SelectionState,
    document_selection: Rc<RefCell<DocumentSelection>>,
    geometry: Rc<RefCell<DocumentGeometry>>,
    scroll_offset: Rc<Cell<f32>>,
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

        let document_selection = Rc::new(RefCell::new(DocumentSelection::default()));
        let geometry = Rc::new(RefCell::new(DocumentGeometry::default()));
        let scroll_offset = Rc::new(Cell::new(0.0));
        let viewer = Viewer::new(
            config.clone(),
            fonts.clone(),
            Rc::clone(&document_selection),
            Rc::clone(&geometry),
        );
        let program = Self {
            config: config.clone(),
            fonts,
            content,
            watcher,
            viewer,
            startup,
            gpu: GpuState::Unknown,
            dialog: Dialog::default(),
            selection: SelectionState::default(),
            document_selection,
            geometry,
            scroll_offset,
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

    pub(super) fn needs_redraw_event(&self) -> bool {
        self.startup.is_some() || matches!(self.gpu, GpuState::Unknown | GpuState::Checking)
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
        self.scroll_offset.set(0.0);
        self.reset_selections();
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
        self.reset_selections();
    }

    pub(super) fn scroll(&self) -> AbsoluteOffset {
        AbsoluteOffset {
            x: 0.0,
            y: self.scroll_offset.get(),
        }
    }

    pub(super) fn selection(&self) -> Option<&str> {
        self.selection.text()
    }

    pub(super) fn set_selection(&mut self, selection: String) {
        self.selection.set(selection);
    }

    pub(super) fn clear_copy(&mut self) {
        self.selection.clear();
    }

    pub(super) fn clear_selections(&mut self) {
        self.selection.clear();
        self.document_selection.borrow_mut().clear();
    }

    pub(super) fn reset_selections(&mut self) {
        self.selection.reset();
        let generation = self.selection.generation();
        self.document_selection
            .borrow_mut()
            .sync_generation(generation);
        self.geometry.borrow_mut().reset(generation);
    }

    pub(super) fn generation(&self) -> crate::selection::Generation {
        self.selection.generation()
    }

    pub(super) fn document_selection(&self) -> Rc<RefCell<DocumentSelection>> {
        Rc::clone(&self.document_selection)
    }

    pub(super) fn geometry(&self) -> Rc<RefCell<DocumentGeometry>> {
        Rc::clone(&self.geometry)
    }

    pub(super) fn scroll_offset(&self) -> Rc<Cell<f32>> {
        Rc::clone(&self.scroll_offset)
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
        let generation = self.selection.generation();
        self.viewer.set_generation(generation);
        self.document_selection
            .borrow_mut()
            .sync_generation(generation);
        self.geometry.borrow_mut().reset(generation);
    }
}

fn argument_path() -> Option<PathBuf> {
    std::env::args_os().nth(1).map(PathBuf::from)
}
