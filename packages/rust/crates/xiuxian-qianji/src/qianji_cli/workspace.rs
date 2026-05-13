use std::env;
use std::io;
use std::path::{Path, PathBuf};

use xiuxian_config_core::resolve_project_root_or_cwd_from_value;

use super::input::resolve_path_against_root;

pub(crate) fn resolve_workspace_root(explicit: Option<&Path>) -> io::Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let raw_project_root = env::var("PRJ_ROOT").ok();
    let base = explicit.map_or_else(
        || {
            resolve_project_root_or_cwd_from_value(
                raw_project_root.as_deref(),
                Some(current_dir.as_path()),
            )
        },
        Path::to_path_buf,
    );

    Ok(resolve_path_against_root(base, current_dir.as_path()))
}
