use std::path::{Path, PathBuf};

use crate::document::Document;
use crate::markdown::Article;

#[derive(Debug)]
pub enum Content {
    Empty,
    Ready {
        document: Document,
        article: Article,
    },
    Failed(PathBuf),
}

impl Content {
    pub fn open(path: PathBuf) -> Self {
        match Document::new(path.clone()) {
            Some(document) => ready(document),
            None => Self::Failed(path),
        }
    }

    pub fn reload(self) -> Self {
        let Some(path) = self.path().map(Path::to_path_buf) else {
            return self;
        };

        match Document::new(path) {
            Some(document) => ready(document),
            None => self,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Ready { document, .. } => Some(document.path()),
            Self::Failed(path) => Some(path),
            Self::Empty => None,
        }
    }

    pub fn article(&self) -> Option<&Article> {
        match self {
            Self::Ready { article, .. } => Some(article),
            Self::Failed(_) | Self::Empty => None,
        }
    }

    pub fn failed_path(&self) -> Option<&Path> {
        match self {
            Self::Failed(path) => Some(path),
            Self::Ready { .. } | Self::Empty => None,
        }
    }
}

fn ready(document: Document) -> Content {
    let base = document.path().parent().unwrap_or_else(|| Path::new(""));
    let source = crate::markdown::resolve(document.source(), base);
    let article = Article::parse(&source);

    Content::Ready { document, article }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temporary(name: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("mdrs-content-{}-{name}", std::process::id()));
        fs::write(&path, content).expect("fixture file");
        path
    }

    #[test]
    fn existing_file_opens_ready() {
        let path = temporary("ready.md", "# title\n\nbody");

        let content = Content::open(path.clone());

        assert!(matches!(content, Content::Ready { .. }));
        assert_eq!(content.path(), Some(path.as_path()));
        assert!(!content.article().expect("article").items().is_empty());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn missing_path_opens_failed_with_path() {
        let path =
            std::env::temp_dir().join(format!("mdrs-content-{}-missing.md", std::process::id()));

        let content = Content::open(path.clone());

        assert!(matches!(content, Content::Failed(_)));
        assert_eq!(content.failed_path(), Some(path.as_path()));
        assert_eq!(content.path(), Some(path.as_path()));
        assert!(content.article().is_none());
    }

    #[test]
    fn reload_keeps_last_content_when_file_disappears() {
        let path = temporary("reload.md", "# title");

        let content = Content::open(path.clone());
        fs::remove_file(&path).expect("remove file");
        let reloaded = content.reload();

        assert!(matches!(reloaded, Content::Ready { .. }));
        assert_eq!(reloaded.path(), Some(path.as_path()));
        assert!(!reloaded.article().expect("article").items().is_empty());
    }
}
