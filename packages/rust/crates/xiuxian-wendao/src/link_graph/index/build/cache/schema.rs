//! `link_graph::index::build::cache::schema` owns Wendao build cache schema behavior.

use crate::schemas::LINK_GRAPH_CACHE_SNAPSHOT_V1;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::OnceLock;

static LINK_GRAPH_CACHE_SCHEMA_FINGERPRINT: OnceLock<String> = OnceLock::new();
const LINK_GRAPH_CACHE_SCHEMA_JSON: &str = LINK_GRAPH_CACHE_SNAPSHOT_V1;
const LINK_GRAPH_CACHE_INDEXING_CONTRACT_REVISION: &str = "semantic_frontmatter_relation_search_v2";

/// Schema version identifier for persisted `LinkGraph` cache snapshots.
pub const LINK_GRAPH_CACHE_SCHEMA_VERSION: &str = "xiuxian_wendao.link_graph.cache_snapshot.v1";
/// `cache_schema_fingerprint` public function boundary for Wendao.
pub fn cache_schema_fingerprint() -> &'static str {
    LINK_GRAPH_CACHE_SCHEMA_FINGERPRINT.get_or_init(|| {
        let mut hasher = DefaultHasher::new();
        LINK_GRAPH_CACHE_SCHEMA_JSON.hash(&mut hasher);
        LINK_GRAPH_CACHE_INDEXING_CONTRACT_REVISION.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    })
}
/// `cache_slot_key` public function boundary for Wendao.
pub fn cache_slot_key(root: &Path, include_dirs: &[String], excluded_dirs: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    root.hash(&mut hasher);
    include_dirs.hash(&mut hasher);
    excluded_dirs.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
