use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

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
    ensure_workspace_wendao_binary();
    Command::new(wendao_binary_path())
}

fn ensure_workspace_wendao_binary() {
    if std::env::var_os("CARGO_BIN_EXE_wendao").is_some() {
        return;
    }
    if target_sibling_binary("wendao")
        .as_deref()
        .is_some_and(wendao_binary_is_current)
    {
        return;
    }

    let result = WENDAO_BINARY_BUILD.get_or_init(build_workspace_wendao_binary);
    if let Err(message) = result {
        panic!("{message}");
    }
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

fn wendao_binary_is_current(path: &Path) -> bool {
    let Ok(binary_modified) = fs::metadata(path).and_then(|metadata| metadata.modified()) else {
        return false;
    };
    let Ok(test_modified) = std::env::current_exe()
        .and_then(|path| fs::metadata(path).and_then(|metadata| metadata.modified()))
    else {
        return false;
    };
    binary_modified >= test_modified
}

static WENDAO_BINARY_BUILD: OnceLock<Result<(), String>> = OnceLock::new();

fn build_workspace_wendao_binary() -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(workspace_root.as_path())
        .args([
            "build",
            "-p",
            "xiuxian-wendao-studio",
            "--features",
            "cli-bin-support zhenfa-router julia",
            "--bin",
            "wendao",
        ])
        .status()
        .map_err(|error| format!("failed to build test `wendao` binary: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("test `wendao` binary build failed with {status}"))
    }
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(4)
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| {
            format!(
                "failed to resolve workspace root from `{}`",
                manifest_dir.display()
            )
        })
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
