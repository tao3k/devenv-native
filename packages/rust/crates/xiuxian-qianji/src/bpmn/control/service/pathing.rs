use std::env;
use std::io;
use std::path::{Path, PathBuf};

pub(super) fn resolve_path_against_current_dir(path: &Path) -> io::Result<PathBuf> {
    Ok(resolve_path_against_root(
        path.to_path_buf(),
        &env::current_dir()?,
    ))
}

fn resolve_path_against_root(path: PathBuf, root: &Path) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}
