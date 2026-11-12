use crate::config::node::Node;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wheel {
    pub smoothing: f32,
    pub settle_distance: f32,
    pub lines_per_notch: f32,
    pub pixels_per_notch: f32,
    pub line_height: f32,
    pub external_distance: f32,
}

impl Default for Wheel {
    fn default() -> Self {
        Self {
            smoothing: 0.03,
            settle_distance: 0.5,
            lines_per_notch: 8.0,
            pixels_per_notch: 15.0,
            line_height: 24.0,
            external_distance: 0.5,
        }
    }
}

impl Wheel {
    fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            smoothing: node.number_child("smoothing").unwrap_or(base.smoothing),
            settle_distance: node
                .number_child("settle-distance")
                .unwrap_or(base.settle_distance),
            lines_per_notch: node
                .number_child("lines-per-notch")
                .unwrap_or(base.lines_per_notch),
            pixels_per_notch: node
                .number_child("pixels-per-notch")
                .unwrap_or(base.pixels_per_notch),
            line_height: node.number_child("line-height").unwrap_or(base.line_height),
            external_distance: node
                .number_child("external-distance")
                .unwrap_or(base.external_distance),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Motion {
    pub wheel: Wheel,
}

impl Motion {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            wheel: Wheel::from_node(base.wheel, node.child("wheel")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn wheel_step_is_configurable() {
        let document = Document::parse(
            r#"
motion {
    wheel {
        lines-per-notch 4
        line-height 30
    }
}
"#,
        )
        .expect("document");
        let motion = Motion::from_node(Motion::default(), document.root().child("motion"));

        assert_eq!(motion.wheel.lines_per_notch, 4.0);
        assert_eq!(motion.wheel.line_height, 30.0);
        assert_eq!(motion.wheel.smoothing, 0.03);
    }
}
