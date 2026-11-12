mod decoration;
mod generation;
mod inline_atom;
mod inline_atomize;
mod inline_flow;
mod inline_layout;
mod inline_render;
mod inline_state;
#[allow(clippy::module_inception)]
mod selectable_text;
mod selection;
mod state;
mod text_layout;

pub use decoration::Decoration;
pub use generation::Generation;
pub use inline_atom::InlineConfig;
pub use inline_flow::InlineFlow;
pub use selectable_text::{SelectableText, TextColors};
pub use selection::Selection;
