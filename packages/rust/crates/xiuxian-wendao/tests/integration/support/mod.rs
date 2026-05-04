use std::path::PathBuf;
use std::process::Command;

#[path = "../../support/linked_parser_summary.rs"]
pub(crate) mod linked_parser_summary;
#[path = "../../support/repo_fixture.rs"]
pub(crate) mod repo_fixture;
#[path = "../../support/repo_intelligence.rs"]
pub(crate) mod repo_intelligence;
#[path = "../../support/repo_parser_summary/mod.rs"]
pub(crate) mod repo_parser_summary;
#[path = "../../support/repo_projection_support.rs"]
pub(crate) mod repo_projection_support;

pub(crate) fn wendao_command() -> Command {
    Command::new(wendao_binary_path())
}

fn wendao_binary_path() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_wendao") {
        return PathBuf::from(path);
    }
    let Some(path) = target_sibling_binary("wendao") else {
        return PathBuf::from("wendao");
    };
    if path.is_file() {
        return path;
    }
    PathBuf::from("wendao")
}

fn target_sibling_binary(name: &str) -> Option<PathBuf> {
    let mut dir = std::env::current_exe().ok()?;
    dir.pop();
    if dir
        .file_name()
        .is_some_and(|file_name| file_name == std::ffi::OsStr::new("deps"))
    {
        dir.pop();
    }
    Some(dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX)))
}
