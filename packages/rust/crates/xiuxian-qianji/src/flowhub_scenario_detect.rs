use std::path::Path;

use crate::flowhub::load_flowhub_scenario_manifest;

/// Returns `true` when `dir/qianji.toml` parses as a Flowhub scenario manifest.
///
/// Invalid or unrelated `qianji.toml` files return `false` so the caller can
/// continue probing other bounded surfaces.
#[must_use]
pub fn looks_like_flowhub_scenario_dir(dir: impl AsRef<Path>) -> bool {
    let dir = dir.as_ref();
    let manifest_path = dir.join("qianji.toml");
    if !manifest_path.is_file() {
        return false;
    }

    load_flowhub_scenario_manifest(&manifest_path).is_ok()
}
