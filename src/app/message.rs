use std::path::PathBuf;
use std::time::Instant;

use iced::widget::markdown::Uri;

use crate::app::Gpu;
use crate::theme::{CodeCopy, CopyHover};

#[derive(Debug, Clone)]
pub enum Message {
    Choose,
    ClearSelections,
    CopyCode { code: String, id: u64 },
    CopyHover { id: u64, hovered: bool },
    AnimationFrame(Instant),
    CopySelection,
    Dropped(PathBuf),
    Gpu(Gpu),
    Link(Uri),
    Picked(Option<PathBuf>),
    Redrawn,
    Reload,
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

impl From<CodeCopy> for Message {
    fn from(copy: CodeCopy) -> Self {
        Self::CopyCode {
            code: copy.code,
            id: copy.id,
        }
    }
}

impl From<CopyHover> for Message {
    fn from(hover: CopyHover) -> Self {
        Self::CopyHover {
            id: hover.id,
            hovered: hover.hovered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_copy_becomes_copy_message() {
        match Message::from(CodeCopy {
            code: "let value = 1;".to_owned(),
            id: 7,
        }) {
            Message::CopyCode { code, id } => {
                assert_eq!(code, "let value = 1;");
                assert_eq!(id, 7);
            }
            _ => panic!("unexpected message"),
        }
    }
}
