//! `analyzers::service::search::ranking::symbol` owns Wendao search ranking symbol behavior.

#[cfg(feature = "repo-lexical-index")]
use std::collections::BTreeMap;
use std::collections::HashSet;

use crate::analyzers::SymbolRecord;
use crate::analyzers::service::normalized_rank_score;
#[cfg(feature = "repo-lexical-index")]
use crate::search::SearchDocumentIndex;

#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::documents::{
    build_search_document_index, symbol_search_document,
};
#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::indexed_exact::{
    indexed_symbol_exact_matches, indexed_symbol_prefix_matches,
};
#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::indexed_fuzzy::indexed_symbol_fuzzy_matches;
use crate::analyzers::service::search::legacy::legacy_symbol_matches;

#[cfg(feature = "repo-lexical-index")]
use super::search_candidate_limit;
use super::{RankedSearchRecord, SYMBOL_SEARCH_BUCKETS};

pub(crate) fn ranked_symbol_matches(
    query: &str,
    symbols: &[SymbolRecord],
    limit: usize,
) -> Vec<RankedSearchRecord<SymbolRecord>> {
    #[cfg(feature = "repo-lexical-index")]
    {
        let lookup = symbols
            .iter()
            .map(|symbol| (symbol.symbol_id.to_string(), symbol.clone()))
            .collect::<BTreeMap<_, _>>();
        let Some(index) = build_search_document_index(symbols.iter().map(symbol_search_document))
        else {
            return ranked_symbol_matches_without_index(query, symbols, limit);
        };
        ranked_symbol_matches_from_index(query, symbols, &lookup, &index, limit)
    }

    #[cfg(not(feature = "repo-lexical-index"))]
    {
        ranked_symbol_matches_without_index(query, symbols, limit)
    }
}
/// `ranked_symbol_matches_with_artifacts` public function boundary for Wendao.

#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn ranked_symbol_matches_with_artifacts(
    query: &str,
    symbols: &[SymbolRecord],
    lookup: &BTreeMap<String, SymbolRecord>,
    index: &SearchDocumentIndex,
    limit: usize,
) -> Vec<RankedSearchRecord<SymbolRecord>> {
    ranked_symbol_matches_from_index(query, symbols, lookup, index, limit)
}

#[cfg(feature = "repo-lexical-index")]
fn ranked_symbol_matches_from_index(
    query: &str,
    symbols: &[SymbolRecord],
    lookup: &BTreeMap<String, SymbolRecord>,
    index: &SearchDocumentIndex,
    limit: usize,
) -> Vec<RankedSearchRecord<SymbolRecord>> {
    if symbols.is_empty() || limit == 0 {
        return Vec::new();
    }

    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return symbols
            .iter()
            .take(limit)
            .cloned()
            .map(|symbol| RankedSearchRecord {
                item: symbol,
                score: normalized_rank_score(0, SYMBOL_SEARCH_BUCKETS),
            })
            .collect();
    }

    let search_limit = search_candidate_limit(limit);
    let mut ranked = Vec::new();
    let mut seen_ids = HashSet::new();

    for candidate in indexed_symbol_exact_matches(
        index,
        lookup,
        query,
        normalized_query.as_str(),
        search_limit,
    ) {
        if seen_ids.insert(candidate.item.symbol_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in indexed_symbol_prefix_matches(
        index,
        lookup,
        query,
        normalized_query.as_str(),
        search_limit,
    ) {
        if seen_ids.insert(candidate.item.symbol_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in indexed_symbol_fuzzy_matches(index, lookup, query, search_limit) {
        if seen_ids.insert(candidate.item.symbol_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in legacy_symbol_matches(normalized_query.as_str(), symbols) {
        if seen_ids.insert(candidate.item.symbol_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                break;
            }
        }
    }

    ranked
}

fn ranked_symbol_matches_without_index(
    query: &str,
    symbols: &[SymbolRecord],
    limit: usize,
) -> Vec<RankedSearchRecord<SymbolRecord>> {
    if symbols.is_empty() || limit == 0 {
        return Vec::new();
    }

    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return symbols
            .iter()
            .take(limit)
            .cloned()
            .map(|symbol| RankedSearchRecord {
                item: symbol,
                score: normalized_rank_score(0, SYMBOL_SEARCH_BUCKETS),
            })
            .collect();
    }

    let mut ranked = Vec::new();
    let mut seen_ids = HashSet::new();
    for candidate in legacy_symbol_matches(normalized_query.as_str(), symbols) {
        if seen_ids.insert(candidate.item.symbol_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                break;
            }
        }
    }
    ranked
}
