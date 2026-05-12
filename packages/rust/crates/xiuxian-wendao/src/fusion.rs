//! Fusion Recall Boost — high-performance Rust implementation.
//!
//! Pure computation: apply `LinkGraph` link/tag proximity boost to recall results.
//! `Python` provides a thin wrapper (`LinkGraph` data fetch); all score computation runs here.

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

#[derive(Debug, Clone, Copy)]
struct PairBoost {
    left: usize,
    right: usize,
    link: bool,
    tag: bool,
}

/// Apply `LinkGraph` link and tag proximity boost to recall results.
///
/// For each pair of results (`i`, `j`) where stems share a `LinkGraph` link or tag:
/// - Add `link_boost` to both scores when stems are bidirectionally linked
/// - Add `tag_boost` to both scores when stems share tags
///
/// Results are re-sorted by score (descending) in place.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn apply_link_graph_proximity_boost<LinksHasher, LinkSetHasher, TagsHasher, TagSetHasher>(
    results: &mut [RecallResult],
    stem_links: &HashMap<String, HashSet<String, LinkSetHasher>, LinksHasher>,
    stem_tags: &HashMap<String, HashSet<String, TagSetHasher>, TagsHasher>,
    link_boost: f64,
    tag_boost: f64,
) where
    LinksHasher: BuildHasher,
    LinkSetHasher: BuildHasher,
    TagsHasher: BuildHasher,
    TagSetHasher: BuildHasher,
{
    if results.len() < 2 {
        return;
    }

    let stems = result_stems(results);
    let boosts = collect_pair_boosts(&stems, stem_links, stem_tags);
    apply_pair_boosts(results, &boosts, link_boost, tag_boost);
    sort_results_by_score(results);
}

/// Extract stem from source path (filename without extension).
pub fn stem_from_source(source: &str) -> String {
    source
        .rsplit('/')
        .next()
        .unwrap_or(source)
        .rsplit('.')
        .nth(1)
        .map_or_else(|| source.to_string(), std::string::ToString::to_string)
}

/// Recall result for boost computation.
#[derive(Debug, Clone)]
pub struct RecallResult {
    /// Source identifier (usually a file path).
    pub source: String,
    /// Recall score (e.g. cosine similarity or BM25).
    pub score: f64,
    /// Raw text content of the result.
    pub content: String,
    /// Human-readable title.
    pub title: String,
}

impl RecallResult {
    /// Create a new recall result.
    #[must_use]
    pub fn new(source: String, score: f64, content: String, title: String) -> Self {
        Self {
            source,
            score,
            content,
            title,
        }
    }
}

fn result_stems(results: &[RecallResult]) -> Vec<String> {
    results
        .iter()
        .map(|result| stem_from_source(&result.source))
        .collect()
}

fn collect_pair_boosts<LinksHasher, LinkSetHasher, TagsHasher, TagSetHasher>(
    stems: &[String],
    stem_links: &HashMap<String, HashSet<String, LinkSetHasher>, LinksHasher>,
    stem_tags: &HashMap<String, HashSet<String, TagSetHasher>, TagsHasher>,
) -> Vec<PairBoost>
where
    LinksHasher: BuildHasher,
    LinkSetHasher: BuildHasher,
    TagsHasher: BuildHasher,
    TagSetHasher: BuildHasher,
{
    (0..stems.len())
        .flat_map(|left| {
            ((left + 1)..stems.len())
                .filter_map(move |right| pair_boost(left, right, stems, stem_links, stem_tags))
        })
        .collect()
}

fn pair_boost<LinksHasher, LinkSetHasher, TagsHasher, TagSetHasher>(
    left: usize,
    right: usize,
    stems: &[String],
    stem_links: &HashMap<String, HashSet<String, LinkSetHasher>, LinksHasher>,
    stem_tags: &HashMap<String, HashSet<String, TagSetHasher>, TagsHasher>,
) -> Option<PairBoost>
where
    LinksHasher: BuildHasher,
    LinkSetHasher: BuildHasher,
    TagsHasher: BuildHasher,
    TagSetHasher: BuildHasher,
{
    let stem1 = &stems[left];
    let stem2 = &stems[right];
    let links1 = stem_links.get(stem1)?;
    let links2 = stem_links.get(stem2)?;
    let link = links1.contains(stem2) || links2.contains(stem1);
    let tag = stem_tags
        .get(stem1)
        .zip(stem_tags.get(stem2))
        .is_some_and(|(tags1, tags2)| !tags1.is_disjoint(tags2));

    (link || tag).then_some(PairBoost {
        left,
        right,
        link,
        tag,
    })
}

fn apply_pair_boosts(
    results: &mut [RecallResult],
    boosts: &[PairBoost],
    link_boost: f64,
    tag_boost: f64,
) {
    for boost in boosts {
        if boost.link {
            results[boost.left].score += link_boost;
            results[boost.right].score += link_boost;
        }
        if boost.tag {
            results[boost.left].score += tag_boost;
            results[boost.right].score += tag_boost;
        }
    }
}

fn sort_results_by_score(results: &mut [RecallResult]) {
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
#[path = "../tests/unit/fusion.rs"]
mod tests;
