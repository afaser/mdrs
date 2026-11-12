use kdl::KdlDocument;

use crate::config::node::Node;

#[derive(Debug, Clone)]
pub struct Document {
    root: KdlDocument,
}

impl Document {
    pub fn parse(text: &str) -> Option<Self> {
        match text.parse::<KdlDocument>() {
            Ok(root) => Some(Self { root }),
            Err(error) => {
                log::warn!("config: {error}");
                None
            }
        }
    }

    pub fn root(&self) -> Node<'_> {
        Node::root(&self.root)
    }
}
