use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::analyzers::{ExampleRecord, ModuleRecord, SymbolRecord};
use crate::search::{FuzzySearchOptions, SearchDocumentIndex};

use super::ranking::RankedSearchRecord;

pub(super) fn indexed_module_fuzzy_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, ModuleRecord>,
    query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    let mut matches = index
        .search_fuzzy_hits(query, limit, FuzzySearchOptions::path_search())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let module = lookup.get(hit.id.as_str())?.clone();
            Some((f64::from(hit.score), hit.distance, module))
        })
        .collect::<Vec<_>>();

    matches.sort_by(
        |(left_score, left_distance, left_module), (right_score, right_distance, right_module)| {
            right_score
                .partial_cmp(left_score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left_distance.cmp(right_distance))
                .then_with(|| left_module.qualified_name.cmp(&right_module.qualified_name))
                .then_with(|| left_module.path.cmp(&right_module.path))
        },
    );

    matches
        .into_iter()
        .map(|(score, _distance, module)| RankedSearchRecord {
            item: module,
            score,
        })
        .collect()
}

pub(super) fn indexed_symbol_fuzzy_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, SymbolRecord>,
    query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<SymbolRecord>> {
    let mut matches = index
        .search_fuzzy_hits(query, limit, FuzzySearchOptions::symbol_search())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let symbol = lookup.get(hit.id.as_str())?.clone();
            Some((f64::from(hit.score), hit.distance, symbol))
        })
        .collect::<Vec<_>>();

    matches.sort_by(
        |(left_score, left_distance, left_symbol), (right_score, right_distance, right_symbol)| {
            right_score
                .partial_cmp(left_score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left_distance.cmp(right_distance))
                .then_with(|| left_symbol.name.cmp(&right_symbol.name))
                .then_with(|| left_symbol.qualified_name.cmp(&right_symbol.qualified_name))
                .then_with(|| left_symbol.path.cmp(&right_symbol.path))
        },
    );

    matches
        .into_iter()
        .map(|(score, _distance, symbol)| RankedSearchRecord {
            item: symbol,
            score,
        })
        .collect()
}

pub(super) fn indexed_example_fuzzy_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, ExampleRecord>,
    query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<ExampleRecord>> {
    let mut matches = index
        .search_fuzzy_hits(query, limit, FuzzySearchOptions::document_search())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let example = lookup.get(hit.id.as_str())?.clone();
            Some((f64::from(hit.score), hit.distance, example))
        })
        .collect::<Vec<_>>();

    matches.sort_by(
        |(left_score, left_distance, left_example),
         (right_score, right_distance, right_example)| {
            right_score
                .partial_cmp(left_score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left_distance.cmp(right_distance))
                .then_with(|| left_example.title.cmp(&right_example.title))
                .then_with(|| left_example.path.cmp(&right_example.path))
        },
    );

    matches
        .into_iter()
        .map(|(score, _distance, example)| RankedSearchRecord {
            item: example,
            score,
        })
        .collect()
}
