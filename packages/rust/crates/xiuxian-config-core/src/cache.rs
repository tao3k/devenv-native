use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::UNIX_EPOCH;

use crate::{ConfigCascadeSpec, spec::ArrayMergeStrategy};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ResolveCacheKey {
    namespace: String,
    orphan_file: String,
    array_merge_strategy: ArrayMergeStrategy,
    embedded_toml_hash: u64,
    project_root: Option<PathBuf>,
    config_home: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileStamp {
    path: PathBuf,
    exists: bool,
    len: u64,
    modified_unix_nanos: Option<u128>,
}

#[derive(Debug, Clone)]
struct ResolveCacheEntry {
    files: Vec<FileStamp>,
    merged: toml::Value,
}

type ResolveCacheMap = HashMap<ResolveCacheKey, ResolveCacheEntry>;

static RESOLVE_CACHE: OnceLock<RwLock<ResolveCacheMap>> = OnceLock::new();

pub(crate) fn cache_key(
    spec: ConfigCascadeSpec<'_>,
    project_root: Option<&Path>,
    config_home: Option<&Path>,
) -> ResolveCacheKey {
    ResolveCacheKey {
        namespace: spec.namespace.to_string(),
        orphan_file: spec.orphan_file.to_string(),
        array_merge_strategy: spec.array_merge_strategy,
        embedded_toml_hash: hash_text(spec.embedded_toml),
        project_root: project_root.map(Path::to_path_buf),
        config_home: config_home.map(Path::to_path_buf),
    }
}

pub(crate) fn build_file_stamps(paths: &[PathBuf]) -> Vec<FileStamp> {
    paths
        .iter()
        .map(|path| stamp_path(path.as_path()))
        .collect()
}

pub(crate) fn try_get_cached_merged(
    key: &ResolveCacheKey,
    current_files: &[FileStamp],
) -> Option<toml::Value> {
    let guard = read_cache();
    let entry = guard.get(key)?;
    if entry.files == current_files {
        return Some(entry.merged.clone());
    }
    None
}

pub(crate) fn store_cached_merged(
    key: ResolveCacheKey,
    files: Vec<FileStamp>,
    merged: &toml::Value,
) {
    let mut guard = write_cache();
    guard.insert(
        key,
        ResolveCacheEntry {
            files,
            merged: merged.clone(),
        },
    );
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn stamp_path(path: &Path) -> FileStamp {
    match std::fs::metadata(path) {
        Ok(metadata) => FileStamp {
            path: path.to_path_buf(),
            exists: true,
            len: metadata.len(),
            modified_unix_nanos: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_nanos()),
        },
        Err(_) => FileStamp {
            path: path.to_path_buf(),
            exists: false,
            len: 0,
            modified_unix_nanos: None,
        },
    }
}

fn cache_store() -> &'static RwLock<ResolveCacheMap> {
    RESOLVE_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn read_cache() -> std::sync::RwLockReadGuard<'static, ResolveCacheMap> {
    match cache_store().read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_cache() -> std::sync::RwLockWriteGuard<'static, ResolveCacheMap> {
    match cache_store().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
