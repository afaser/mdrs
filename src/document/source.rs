use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Document {
    path: PathBuf,
    source: String,
}

impl Document {
    pub fn new(path: PathBuf) -> Option<Self> {
        let source = fs::read_to_string(&path).ok()?;

        Some(Self { path, source })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temporary(name: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("mdrs-{}-{name}", std::process::id()));
        fs::write(&path, content).expect("fixture file");
        path
    }

    #[test]
    fn existing_file_loads_source() {
        let path = temporary("existing.md", "# title");

        let document = Document::new(path.clone());

        assert!(document.is_some());
        assert_eq!(document.as_ref().map(Document::source), Some("# title"));
        assert_eq!(document.as_ref().map(Document::path), Some(path.as_path()));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn missing_path_is_none() {
        let path = std::env::temp_dir().join(format!("mdrs-{}-missing.md", std::process::id()));

        let document = Document::new(path);

        assert!(document.is_none());
    }

    #[test]
    fn empty_file_loads_empty_source() {
        let path = temporary("empty.md", "");

        let document = Document::new(path.clone());

        assert_eq!(document.as_ref().map(Document::source), Some(""));
        assert_eq!(document.as_ref().map(Document::path), Some(path.as_path()));

        let _ = fs::remove_file(path);
    }
}
