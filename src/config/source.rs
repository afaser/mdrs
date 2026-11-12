use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    path: PathBuf,
}

impl Source {
    pub fn discover() -> Option<Self> {
        if let Some(path) = std::env::var_os("MDRS_CONFIG").filter(|path| !path.is_empty()) {
            return Some(Self {
                path: PathBuf::from(path),
            });
        }

        let xdg = std::env::var_os("XDG_CONFIG_HOME");
        let home = std::env::var_os("HOME");
        let path = default_path(xdg.as_deref(), home.as_deref())?;

        path.is_file().then_some(Self { path })
    }

    pub fn read(&self) -> Option<String> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => Some(text),
            Err(error) => {
                log::warn!("config: {}: {error}", self.path.display());
                None
            }
        }
    }
}

fn default_path(xdg: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    if let Some(xdg) = xdg.filter(|value| !value.is_empty()) {
        return Some(Path::new(xdg).join("mdrs").join("config.kdl"));
    }

    let home = home.filter(|value| !value.is_empty())?;

    Some(
        Path::new(home)
            .join(".config")
            .join("mdrs")
            .join("config.kdl"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_has_priority_over_home() {
        let path = default_path(Some(OsStr::new("/xdg")), Some(OsStr::new("/home/user")));

        assert_eq!(path, Some(PathBuf::from("/xdg/mdrs/config.kdl")));
    }

    #[test]
    fn home_is_used_without_xdg() {
        let path = default_path(None, Some(OsStr::new("/home/user")));

        assert_eq!(
            path,
            Some(PathBuf::from("/home/user/.config/mdrs/config.kdl"))
        );
    }

    #[test]
    fn missing_environment_has_no_path() {
        assert_eq!(default_path(None, None), None);
    }

    #[test]
    fn read_returns_none_for_missing_file() {
        let source = Source {
            path: PathBuf::from("/definitely/missing/config.kdl"),
        };

        assert_eq!(source.read(), None);
    }
}
