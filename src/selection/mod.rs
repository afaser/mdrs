mod block_source;
mod decoration;
mod document_geometry;
mod document_selection;
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
mod selection_surface;
mod state;
mod text_layout;

pub use block_source::BlockSource;
pub use decoration::Decoration;
pub use document_geometry::DocumentGeometry;
pub use document_selection::{DocumentSelection, SelectionMode};
pub use generation::Generation;
pub use inline_atom::InlineConfig;
pub use inline_flow::InlineFlow;
pub use selectable_text::{SelectableText, TextColors};
pub use selection::Selection;
pub use selection_surface::SelectionSurface;
