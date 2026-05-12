//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! `link_graph::index::shared` owns Wendao link graph index shared behavior.

use super::{
    LinkGraphDocument, LinkGraphHit, LinkGraphSortField, LinkGraphSortOrder, LinkGraphSortTerm,
};
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(super) fn doc_sort_key(doc: &LinkGraphDocument) -> (&str, &str) {
    (doc.path.as_str(), doc.stem.as_str())
}
/// `ScoredSearchRow` public type boundary for Wendao.

#[derive(Debug, Clone)]
pub struct ScoredSearchRow {
    pub hit: LinkGraphHit,
    pub created_ts: Option<i64>,
    pub modified_ts: Option<i64>,
    pub word_count: usize,
    pub random_key: u64,
}

fn cmp_optional_ts_asc(left: Option<i64>, right: Option<i64>) -> Ordering {
    match (left, right) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_by_sort_term(
    left: &ScoredSearchRow,
    right: &ScoredSearchRow,
    term: LinkGraphSortTerm,
) -> Ordering {
    let base = match term.field {
        LinkGraphSortField::Score => left
            .hit
            .score
            .partial_cmp(&right.hit.score)
            .unwrap_or(Ordering::Equal),
        LinkGraphSortField::Path => left.hit.path.cmp(&right.hit.path),
        LinkGraphSortField::Title => left.hit.title.cmp(&right.hit.title),
        LinkGraphSortField::Stem => left.hit.stem.cmp(&right.hit.stem),
        LinkGraphSortField::Created => cmp_optional_ts_asc(left.created_ts, right.created_ts),
        LinkGraphSortField::Modified => cmp_optional_ts_asc(left.modified_ts, right.modified_ts),
        LinkGraphSortField::Random => left.random_key.cmp(&right.random_key),
        LinkGraphSortField::WordCount => left.word_count.cmp(&right.word_count),
    };
    match term.order {
        LinkGraphSortOrder::Asc => base,
        LinkGraphSortOrder::Desc => base.reverse(),
    }
}
/// `sort_hits` public function boundary for Wendao.

pub fn sort_hits(rows: &mut [ScoredSearchRow], sort_terms: &[LinkGraphSortTerm]) {
    let terms = if sort_terms.is_empty() {
        vec![LinkGraphSortTerm::default()]
    } else {
        sort_terms.to_vec()
    };
    rows.sort_by(|left, right| {
        for term in &terms {
            let ord = compare_by_sort_term(left, right, *term);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        right
            .hit
            .score
            .partial_cmp(&left.hit.score)
            .unwrap_or(Ordering::Equal)
            .then(left.hit.path.cmp(&right.hit.path))
            .then(left.hit.stem.cmp(&right.hit.stem))
    });
}
/// `deterministic_random_key` public function boundary for Wendao.

pub fn deterministic_random_key(stem: &str, path: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    stem.hash(&mut hasher);
    path.hash(&mut hasher);
    hasher.finish()
}
/// `normalize_path_filter` public function boundary for Wendao.

pub fn normalize_path_filter(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_lowercase()
}
/// `path_matches_filter` public function boundary for Wendao.

pub fn path_matches_filter(path: &str, filter: &str) -> bool {
    if filter.is_empty() {
        return false;
    }
    let normalized_path = normalize_path_filter(path);
    normalized_path == filter || normalized_path.starts_with(&format!("{filter}/"))
}
/// `doc_contains_phrase` public function boundary for Wendao.

pub fn doc_contains_phrase(doc: &LinkGraphDocument, phrase: &str, case_sensitive: bool) -> bool {
    if phrase.trim().is_empty() {
        return false;
    }
    if case_sensitive {
        doc.search_text.contains(phrase)
    } else {
        doc.search_text_lower.contains(&phrase.to_lowercase())
    }
}
