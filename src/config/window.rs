use crate::config::node::Node;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window {
    pub width: f32,
    pub height: f32,
    pub antialiasing: bool,
    pub transparent: Option<bool>,
    pub padding: f32,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 750.0,
            antialiasing: true,
            transparent: None,
            padding: 24.0,
        }
    }
}

impl Window {
    pub fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            width: node.number_child("width").unwrap_or(base.width),
            height: node.number_child("height").unwrap_or(base.height),
            antialiasing: node
                .boolean_child("antialiasing")
                .unwrap_or(base.antialiasing),
            transparent: node.boolean_child("transparent"),
            padding: node.number_child("padding").unwrap_or(base.padding),
        }
    }

    pub fn transparent_for(self, background_alpha: f32) -> bool {
        self.transparent.unwrap_or(background_alpha < 1.0)
    }

    pub fn size(self) -> iced::Size {
        iced::Size::new(self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::document::Document;

    #[test]
    fn window_size_is_configurable() {
        let document = Document::parse("window { width 1200; height 800 }").expect("document");
        let window = Window::from_node(Window::default(), document.root().child("window"));

        assert_eq!(window.width, 1200.0);
        assert_eq!(window.height, 800.0);
        assert_eq!(window.size(), iced::Size::new(1200.0, 800.0));
    }

    #[test]
    fn transparency_derives_from_background_alpha() {
        assert!(!Window::default().transparent_for(1.0));
        assert!(Window::default().transparent_for(0.9));
        assert!(
            !Window {
                transparent: Some(false),
                ..Window::default()
            }
            .transparent_for(0.9)
        );
    }
}
