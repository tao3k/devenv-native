use super::{expand_home_path, resolve_home_dir};
use std::path::{Path, PathBuf};

#[test]
fn expand_home_path_uses_provided_home_dir() {
    let expanded = expand_home_path("~/config/wendao.toml", Some(Path::new("/tmp/home")));
    assert_eq!(expanded, PathBuf::from("/tmp/home/config/wendao.toml"));
}

#[test]
fn expand_home_path_keeps_literal_path_without_home_dir() {
    let expanded = expand_home_path("~/config/wendao.toml", None);
    assert_eq!(expanded, PathBuf::from("~/config/wendao.toml"));
}

#[test]
fn expand_home_path_keeps_non_tilde_paths() {
    let expanded = expand_home_path("./config/wendao.toml", Some(Path::new("/tmp/home")));
    assert_eq!(expanded, PathBuf::from("./config/wendao.toml"));
}

#[test]
fn resolve_home_dir_prefers_current_process_environment() {
    let expected = std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| {
            let drive = std::env::var_os("HOMEDRIVE")?;
            let path = std::env::var_os("HOMEPATH")?;
            let mut combined = PathBuf::from(drive);
            combined.push(path);
            Some(combined)
        });
    assert_eq!(resolve_home_dir(), expected);
}
