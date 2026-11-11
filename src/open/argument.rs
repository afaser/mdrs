use std::path::PathBuf;

pub fn path() -> Option<PathBuf> {
    std::env::args_os().nth(1).map(PathBuf::from)
}
