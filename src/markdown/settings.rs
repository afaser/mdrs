use iced::{Color, Font, Padding, Pixels};

use crate::markdown::item::HeadingLevel;
use crate::text::Highlight;

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub text_size: Pixels,
    pub h1_size: Pixels,
    pub h2_size: Pixels,
    pub h3_size: Pixels,
    pub h4_size: Pixels,
    pub h5_size: Pixels,
    pub h6_size: Pixels,
    pub code_size: Pixels,
    pub spacing: Pixels,
    pub style: Style,
}

impl Settings {
    pub fn heading_size(&self, level: &HeadingLevel) -> Pixels {
        match level {
            HeadingLevel::H1 => self.h1_size,
            HeadingLevel::H2 => self.h2_size,
            HeadingLevel::H3 => self.h3_size,
            HeadingLevel::H4 => self.h4_size,
            HeadingLevel::H5 => self.h5_size,
            HeadingLevel::H6 => self.h6_size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub font: Font,
    pub inline_code_highlight: Highlight,
    pub inline_code_padding: Padding,
    pub inline_code_color: Color,
    pub inline_code_font: Font,
    pub code_block_font: Font,
    pub link_color: Color,
}
