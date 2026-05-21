//! `zhenfa_router::native::audit::fuzzy_suggest::cache` owns Wendao audit fuzzy suggest cache behavior.

use std::cell::RefCell;
use std::collections::HashMap;

use super::types::CandidateMatch;

/// Minimum confidence threshold for suggesting a pattern fix.
/// Set to 0.65 to allow reasonable renamed symbol detection while filtering out poor matches.
pub(crate) const CONFIDENCE_THRESHOLD: f32 = 0.65;

/// Thread-local cache for candidate matches.
/// Key: `file_path`
/// Value: (`content_hash`, Vec<CandidateMatch>)
type CacheValue = (u64, Vec<CandidateMatch>);

thread_local! {
    static CANDIDATE_CACHE: RefCell<HashMap<String, CacheValue>> = RefCell::new(HashMap::new());
}

/// Simple hash function for content.
pub(crate) fn hash_content(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Clear the candidate cache.
/// Call this when source files have been modified.
pub fn clear_candidate_cache() {
    CANDIDATE_CACHE.with(|cache| cache.borrow_mut().clear());
}

/// Get cache statistics for debugging.
#[must_use]
/// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
pub fn cache_stats() -> (usize, usize) {
    CANDIDATE_CACHE.with(|cache| {
        let c = cache.borrow();
        (c.len(), c.values().map(|v| v.1.len()).sum())
    })
}

pub(crate) fn lookup_cached_candidates(
    file_path: &str,
    content_hash: u64,
) -> Option<Vec<CandidateMatch>> {
    CANDIDATE_CACHE.with(|cache| {
        let cache = cache.borrow();
        if let Some((cached_hash, candidates)) = cache.get(file_path)
            && *cached_hash == content_hash
        {
            return Some(candidates.clone());
        }
        None
    })
}

pub(crate) fn store_cached_candidates(
    file_path: &str,
    content_hash: u64,
    candidates: Vec<CandidateMatch>,
) {
    CANDIDATE_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .insert(file_path.to_string(), (content_hash, candidates));
    });
}
