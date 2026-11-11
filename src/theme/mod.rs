mod catppuccin;
mod font;
mod highlight;
mod palette;
mod style;
mod viewer;

pub use catppuccin::catppuccin;
pub use font::{BYTES, isolate_system_fonts, regular};
pub use palette::DANGER;
pub use style::markdown_style;
pub use viewer::Viewer;
