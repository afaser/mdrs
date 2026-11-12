use crate::config::node::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontChain {
    names: Vec<String>,
}

impl FontChain {
    pub fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn first(&self) -> Option<&str> {
        self.names.first().map(String::as_str)
    }

    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        match Self::optional_from_node(Some(base), node) {
            Some(chain) => chain,
            None => Self::new(Vec::new()),
        }
    }

    pub(crate) fn optional_from_node(base: Option<Self>, node: Option<Node<'_>>) -> Option<Self> {
        let Some(node) = node else {
            return base;
        };
        let names: Vec<String> = node.arguments().into_iter().map(str::to_owned).collect();

        if names.is_empty() {
            return base;
        }

        Some(Self::new(names))
    }
}
