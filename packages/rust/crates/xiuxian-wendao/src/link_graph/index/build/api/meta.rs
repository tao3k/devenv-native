use super::super::super::LinkGraphCacheBuildMeta;
use super::super::cache::{LINK_GRAPH_CACHE_SCHEMA_VERSION, cache_schema_fingerprint};

pub(super) fn build_cache_meta(
    backend: &str,
    status: &str,
    miss_reason: Option<String>,
) -> LinkGraphCacheBuildMeta {
    LinkGraphCacheBuildMeta {
        backend: backend.to_string(),
        status: status.to_string(),
        miss_reason,
        schema_version: LINK_GRAPH_CACHE_SCHEMA_VERSION.to_string(),
        schema_fingerprint: cache_schema_fingerprint().to_string(),
    }
}
