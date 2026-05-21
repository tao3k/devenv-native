use std::env;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) fn parse_flag_value(
    args: &[String],
    index: &mut usize,
    flag: &str,
) -> io::Result<String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| invalid_input(format!("missing value for {flag}")))
}

pub(crate) fn empty_json_object() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

pub(crate) fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

pub(crate) fn resolve_cli_path(path: &Path) -> io::Result<PathBuf> {
    Ok(resolve_path_against_root(
        path.to_path_buf(),
        &env::current_dir()?,
    ))
}

pub(crate) fn resolve_path_against_root(path: PathBuf, root: &Path) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}
