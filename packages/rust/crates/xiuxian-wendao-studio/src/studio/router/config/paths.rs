use std::path::{Path, PathBuf};

use xiuxian_io::PrjDirs;

/// Returns the path to `wendao.toml` for the given config root.
#[must_use]
pub fn studio_wendao_toml_path(config_root: &Path) -> PathBuf {
    config_root.join("wendao.toml")
}

/// Returns the path to the legacy Studio overlay TOML for the given config root.
#[must_use]
pub fn studio_wendao_overlay_toml_path(config_root: &Path) -> PathBuf {
    config_root.join("wendao.studio.overlay.toml")
}

/// Returns the effective Wendao TOML path for Studio-aware loading.
///
/// Legacy overlay files still win on reads so older persisted state remains
/// bootable until the next write collapses it back into the base config.
#[must_use]
pub fn studio_effective_wendao_toml_path(config_root: &Path) -> PathBuf {
    let overlay_path = studio_wendao_overlay_toml_path(config_root);
    if overlay_path.is_file() {
        overlay_path
    } else {
        studio_wendao_toml_path(config_root)
    }
}

/// Resolves the studio config root directory.
#[must_use]
pub fn resolve_studio_config_root(project_root: &Path) -> PathBuf {
    let candidate = PrjDirs::data_home().join("wendao-frontend");
    if candidate.exists() {
        candidate
    } else {
        project_root.to_path_buf()
    }
}
