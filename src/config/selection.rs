use crate::config::colors::ColorRef;
use crate::config::node::Node;
use crate::config::palette::PaletteKey;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionHeadings {
    pub h1: f32,
    pub h2: f32,
    pub h3: f32,
    pub h4: f32,
    pub h5: f32,
    pub h6: f32,
}

impl SelectionHeadings {
    pub fn get(self, level: usize) -> f32 {
        match level {
            0 => self.h1,
            1 => self.h2,
            2 => self.h3,
            3 => self.h4,
            4 => self.h5,
            _ => self.h6,
        }
    }

    fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            h1: node.number_child("h1").unwrap_or(base.h1),
            h2: node.number_child("h2").unwrap_or(base.h2),
            h3: node.number_child("h3").unwrap_or(base.h3),
            h4: node.number_child("h4").unwrap_or(base.h4),
            h5: node.number_child("h5").unwrap_or(base.h5),
            h6: node.number_child("h6").unwrap_or(base.h6),
        }
    }
}

impl Default for SelectionHeadings {
    fn default() -> Self {
        Self {
            h1: 44.0,
            h2: 39.0,
            h3: 33.0,
            h4: 28.0,
            h5: 22.0,
            h6: 22.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection {
    pub color: ColorRef,
    pub radius: f32,
    pub height: f32,
    pub headings: SelectionHeadings,
    pub code_height: f32,
    pub underline_offset: f32,
    pub strikethrough_offset: f32,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            color: ColorRef::Palette(PaletteKey::Selection),
            radius: 3.0,
            height: 22.0,
            headings: SelectionHeadings::default(),
            code_height: 17.0,
            underline_offset: 0.08,
            strikethrough_offset: 0.5,
        }
    }
}

impl Selection {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };
        let decoration = node.child("decoration");

        Self {
            color: node.color_child("color").unwrap_or(base.color),
            radius: node.number_child("radius").unwrap_or(base.radius),
            height: node.number_child("height").unwrap_or(base.height),
            headings: SelectionHeadings::from_node(base.headings, node.child("headings")),
            code_height: node.number_child("code-height").unwrap_or(base.code_height),
            underline_offset: decoration
                .and_then(|decoration| decoration.number_child("underline-offset"))
                .unwrap_or(base.underline_offset),
            strikethrough_offset: decoration
                .and_then(|decoration| decoration.number_child("strikethrough-offset"))
                .unwrap_or(base.strikethrough_offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn selection_color_and_offsets_are_decoded() {
        let document = Document::parse(
            r##"
selection {
    color "#ff0000"
    decoration {
        underline-offset 0.1
        strikethrough-offset 0.6
    }
}
"##,
        )
        .expect("document");
        let selection =
            Selection::from_node(Selection::default(), document.root().child("selection"));

        assert_eq!(selection.color, ColorRef::parse("#ff0000").expect("color"));
        assert_eq!(selection.underline_offset, 0.1);
        assert_eq!(selection.strikethrough_offset, 0.6);
    }

    #[test]
    fn selection_geometry_uses_defaults_and_overrides() {
        let document = Document::parse(
            r#"
selection {
    radius 6
    headings { h1 46; h3 34 }
    code-height 20
}
"#,
        )
        .expect("document");
        let selection =
            Selection::from_node(Selection::default(), document.root().child("selection"));

        assert_eq!(selection.radius, 6.0);
        assert_eq!(selection.height, 22.0);
        assert_eq!(selection.headings.h1, 46.0);
        assert_eq!(selection.headings.h2, 39.0);
        assert_eq!(selection.headings.h3, 34.0);
        assert_eq!(selection.code_height, 20.0);
    }

    #[test]
    fn repeated_geometry_field_keeps_last_value() {
        let document = Document::parse(
            r#"
selection {
    height 24
    height 28
}
"#,
        )
        .expect("document");
        let selection =
            Selection::from_node(Selection::default(), document.root().child("selection"));

        assert_eq!(selection.height, 28.0);
    }

    #[test]
    fn headings_keep_last_repeated_level() {
        let document = Document::parse(
            r#"
selection {
    headings {
        h2 40
        h2 42
    }
}
"#,
        )
        .expect("document");
        let selection =
            Selection::from_node(Selection::default(), document.root().child("selection"));

        assert_eq!(selection.headings.get(1), 42.0);
        assert_eq!(selection.headings.get(0), 44.0);
    }
}
