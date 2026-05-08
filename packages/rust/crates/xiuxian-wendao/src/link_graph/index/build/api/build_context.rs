use crate::link_graph::index::build::cache::cache_slot_key;
use crate::link_graph::index::build::constants::DEFAULT_EXCLUDED_DIR_NAMES;
use crate::link_graph::index::build::filters::{merge_excluded_dirs, normalize_include_dir};
use crate::link_graph::index::build::fingerprint::{LinkGraphFingerprint, scan_note_fingerprint};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct BuildCacheContext {
    pub(super) slot: BuildCacheSlotContext,
    pub(super) fingerprint: LinkGraphFingerprint,
}

pub(super) struct BuildCacheSlotContext {
    pub(super) root: PathBuf,
    pub(super) normalized_include_dirs: Vec<String>,
    pub(super) normalized_excluded_dirs: Vec<String>,
    pub(super) slot_key: String,
}

pub(super) fn prepare_build_cache_slot_context(
    root_dir: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
) -> Result<BuildCacheSlotContext, String> {
    let root = root_dir
        .canonicalize()
        .map_err(|e| format!("invalid notebook root '{}': {e}", root_dir.display()))?;
    if !root.is_dir() {
        return Err(format!(
            "notebook root is not a directory: {}",
            root.display()
        ));
    }

    let normalized_include_dirs: Vec<String> = include_dirs
        .iter()
        .filter_map(|path| normalize_include_dir(path))
        .collect();
    let normalized_excluded_dirs: Vec<String> =
        merge_excluded_dirs(excluded_dirs, DEFAULT_EXCLUDED_DIR_NAMES);
    let slot_key = cache_slot_key(&root, &normalized_include_dirs, &normalized_excluded_dirs);

    Ok(BuildCacheSlotContext {
        root,
        normalized_include_dirs,
        normalized_excluded_dirs,
        slot_key,
    })
}

pub(super) fn prepare_build_cache_context(
    root_dir: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
) -> Result<BuildCacheContext, String> {
    let slot = prepare_build_cache_slot_context(root_dir, include_dirs, excluded_dirs)?;
    let included: HashSet<String> = slot.normalized_include_dirs.iter().cloned().collect();
    let excluded: HashSet<String> = slot.normalized_excluded_dirs.iter().cloned().collect();
    let fingerprint = scan_note_fingerprint(&slot.root, &included, &excluded);

    Ok(BuildCacheContext { slot, fingerprint })
}
