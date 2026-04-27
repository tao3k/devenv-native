use crate::schemas::LINK_GRAPH_CACHE_SNAPSHOT_V1;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::OnceLock;

static LINK_GRAPH_CACHE_SCHEMA_FINGERPRINT: OnceLock<String> = OnceLock::new();
const LINK_GRAPH_CACHE_SCHEMA_JSON: &str = LINK_GRAPH_CACHE_SNAPSHOT_V1;

/// Schema version identifier for persisted `LinkGraph` cache snapshots.
pub const LINK_GRAPH_CACHE_SCHEMA_VERSION: &str = "xiuxian_wendao.link_graph.cache_snapshot.v1";

pub fn cache_schema_fingerprint() -> &'static str {
    LINK_GRAPH_CACHE_SCHEMA_FINGERPRINT.get_or_init(|| {
        let mut hasher = DefaultHasher::new();
        LINK_GRAPH_CACHE_SCHEMA_JSON.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    })
}

pub fn cache_slot_key(root: &Path, include_dirs: &[String], excluded_dirs: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    root.hash(&mut hasher);
    include_dirs.hash(&mut hasher);
    excluded_dirs.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
