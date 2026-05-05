use std::collections::{BTreeMap, HashSet};

use crate::analyzers::ExampleRecord;
use crate::analyzers::service::normalized_rank_score;
#[cfg(feature = "repo-lexical-index")]
use crate::search::SearchDocumentIndex;

use crate::analyzers::service::search::documents::ExampleSearchMetadata;
#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::documents::{
    build_search_document_index, example_search_document,
};
#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::indexed_exact::{
    indexed_example_exact_matches, indexed_example_prefix_matches,
};
#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::indexed_fuzzy::indexed_example_fuzzy_matches;
use crate::analyzers::service::search::legacy::legacy_example_matches;

#[cfg(feature = "repo-lexical-index")]
use super::search_candidate_limit;
use super::{EXAMPLE_SEARCH_BUCKETS, RankedSearchRecord};

pub(crate) fn ranked_example_matches(
    query: &str,
    examples: &[ExampleRecord],
    metadata_lookup: &BTreeMap<String, ExampleSearchMetadata>,
    limit: usize,
) -> Vec<RankedSearchRecord<ExampleRecord>> {
    #[cfg(feature = "repo-lexical-index")]
    {
        let lookup = examples
            .iter()
            .map(|example| (example.example_id.clone(), example.clone()))
            .collect::<BTreeMap<_, _>>();
        let Some(index) = build_search_document_index(examples.iter().map(|example| {
            let metadata = metadata_lookup
                .get(example.example_id.as_str())
                .cloned()
                .unwrap_or_default();
            example_search_document(example, &metadata)
        })) else {
            return ranked_example_matches_without_index(query, examples, metadata_lookup, limit);
        };
        ranked_example_matches_from_index(query, examples, metadata_lookup, &lookup, &index, limit)
    }

    #[cfg(not(feature = "repo-lexical-index"))]
    {
        ranked_example_matches_without_index(query, examples, metadata_lookup, limit)
    }
}

#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub fn ranked_example_matches_with_artifacts(
    query: &str,
    examples: &[ExampleRecord],
    metadata_lookup: &BTreeMap<String, ExampleSearchMetadata>,
    lookup: &BTreeMap<String, ExampleRecord>,
    index: &SearchDocumentIndex,
    limit: usize,
) -> Vec<RankedSearchRecord<ExampleRecord>> {
    ranked_example_matches_from_index(query, examples, metadata_lookup, lookup, index, limit)
}

#[cfg(feature = "repo-lexical-index")]
fn ranked_example_matches_from_index(
    query: &str,
    examples: &[ExampleRecord],
    metadata_lookup: &BTreeMap<String, ExampleSearchMetadata>,
    lookup: &BTreeMap<String, ExampleRecord>,
    index: &SearchDocumentIndex,
    limit: usize,
) -> Vec<RankedSearchRecord<ExampleRecord>> {
    if examples.is_empty() || limit == 0 {
        return Vec::new();
    }

    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return examples
            .iter()
            .take(limit)
            .cloned()
            .map(|example| RankedSearchRecord {
                item: example,
                score: normalized_rank_score(0, EXAMPLE_SEARCH_BUCKETS),
            })
            .collect();
    }

    let search_limit = search_candidate_limit(limit);
    let mut ranked = Vec::new();
    let mut seen_ids = HashSet::new();

    for candidate in indexed_example_exact_matches(
        index,
        lookup,
        metadata_lookup,
        query,
        normalized_query.as_str(),
        search_limit,
    ) {
        if seen_ids.insert(candidate.item.example_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in indexed_example_prefix_matches(
        index,
        lookup,
        metadata_lookup,
        query,
        normalized_query.as_str(),
        search_limit,
    ) {
        if seen_ids.insert(candidate.item.example_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in indexed_example_fuzzy_matches(index, lookup, query, search_limit) {
        if seen_ids.insert(candidate.item.example_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in legacy_example_matches(normalized_query.as_str(), examples, metadata_lookup) {
        if seen_ids.insert(candidate.item.example_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                break;
            }
        }
    }

    ranked
}

fn ranked_example_matches_without_index(
    query: &str,
    examples: &[ExampleRecord],
    metadata_lookup: &BTreeMap<String, ExampleSearchMetadata>,
    limit: usize,
) -> Vec<RankedSearchRecord<ExampleRecord>> {
    if examples.is_empty() || limit == 0 {
        return Vec::new();
    }

    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return examples
            .iter()
            .take(limit)
            .cloned()
            .map(|example| RankedSearchRecord {
                item: example,
                score: normalized_rank_score(0, EXAMPLE_SEARCH_BUCKETS),
            })
            .collect();
    }

    let mut ranked = Vec::new();
    let mut seen_ids = HashSet::new();
    for candidate in legacy_example_matches(normalized_query.as_str(), examples, metadata_lookup) {
        if seen_ids.insert(candidate.item.example_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                break;
            }
        }
    }
    ranked
}
