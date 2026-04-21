use std::path::{Path, PathBuf};

use xiuxian_config_core::resolve_project_root;

pub fn workspace_root() -> PathBuf {
    resolve_project_root().unwrap_or_else(manifest_workspace_root)
}

fn manifest_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap_or_else(|| panic!("qianji manifest dir should resolve to workspace root in tests"))
        .to_path_buf()
}
