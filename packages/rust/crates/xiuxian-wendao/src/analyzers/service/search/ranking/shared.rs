//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
#[derive(Debug, Clone)]
pub(crate) struct RankedSearchRecord<T> {
    pub(crate) item: T,
    pub(crate) score: f64,
}

pub(crate) const MODULE_SEARCH_BUCKETS: u8 = 3;
pub(crate) const SYMBOL_SEARCH_BUCKETS: u8 = 7;
pub(crate) const EXAMPLE_SEARCH_BUCKETS: u8 = 10;
const SEARCH_CANDIDATE_MULTIPLIER: usize = 8;

pub(crate) fn search_candidate_limit(limit: usize) -> usize {
    limit.max(1).saturating_mul(SEARCH_CANDIDATE_MULTIPLIER)
}
