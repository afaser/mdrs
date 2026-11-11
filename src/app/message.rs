use std::path::PathBuf;

use iced::widget::markdown::Uri;
use iced::widget::scrollable::Viewport;

use crate::app::gpu::Gpu;

#[derive(Debug, Clone)]
pub enum Message {
    Choose,
    ClearSelections,
    CopySelection,
    Dropped(PathBuf),
    Gpu(Gpu),
    Link(Uri),
    Picked(Option<PathBuf>),
    Redrawn,
    Reload,
    Scrolled(Viewport),
    Selected(Option<String>),
}

impl From<Uri> for Message {
    fn from(url: Uri) -> Self {
        Self::Link(url)
    }
}

impl From<Option<String>> for Message {
    fn from(selection: Option<String>) -> Self {
        Self::Selected(selection)
    }
}
