use std::collections::BTreeMap;

use crate::analyzers::{ExampleRecord, ModuleRecord, SymbolRecord};
use crate::search::SearchDocumentIndex;

use super::documents::{ExampleSearchMetadata, raw_example_match_score};
use super::ranking::{
    EXAMPLE_SEARCH_BUCKETS, MODULE_SEARCH_BUCKETS, RankedSearchRecord, SYMBOL_SEARCH_BUCKETS,
};
use crate::analyzers::service::{module_match_score, normalized_rank_score, symbol_match_score};

pub(super) fn indexed_module_exact_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, ModuleRecord>,
    query: &str,
    normalized_query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    let mut matches = index
        .search_exact_hits(query, limit)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let module = lookup.get(hit.id.as_str())?.clone();
            let qualified_name = module.qualified_name.to_ascii_lowercase();
            let path = module.path.to_ascii_lowercase();
            let raw_score =
                module_match_score(normalized_query, qualified_name.as_str(), path.as_str())?;
            Some((raw_score, module))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left_module), (right_score, right_module)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_module.qualified_name.cmp(&right_module.qualified_name))
            .then_with(|| left_module.path.cmp(&right_module.path))
    });

    matches
        .into_iter()
        .map(|(raw_score, module)| RankedSearchRecord {
            item: module,
            score: normalized_rank_score(raw_score, MODULE_SEARCH_BUCKETS),
        })
        .collect()
}

pub(super) fn indexed_module_prefix_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, ModuleRecord>,
    query: &str,
    normalized_query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    let mut matches = index
        .search_prefix_hits(query, limit)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let module = lookup.get(hit.id.as_str())?.clone();
            let qualified_name = module.qualified_name.to_ascii_lowercase();
            let path = module.path.to_ascii_lowercase();
            let raw_score =
                module_match_score(normalized_query, qualified_name.as_str(), path.as_str())?;
            Some((raw_score, module))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left_module), (right_score, right_module)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_module.qualified_name.cmp(&right_module.qualified_name))
            .then_with(|| left_module.path.cmp(&right_module.path))
    });

    matches
        .into_iter()
        .map(|(raw_score, module)| RankedSearchRecord {
            item: module,
            score: normalized_rank_score(raw_score, MODULE_SEARCH_BUCKETS),
        })
        .collect()
}

pub(super) fn indexed_symbol_exact_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, SymbolRecord>,
    query: &str,
    normalized_query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<SymbolRecord>> {
    let mut matches = index
        .search_exact_hits(query, limit)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let symbol = lookup.get(hit.id.as_str())?.clone();
            let name = symbol.name.to_ascii_lowercase();
            let qualified_name = symbol.qualified_name.to_ascii_lowercase();
            let path = symbol.path.to_ascii_lowercase();
            let signature = symbol
                .signature
                .as_deref()
                .map(str::to_ascii_lowercase)
                .unwrap_or_default();
            let raw_score = symbol_match_score(
                normalized_query,
                name.as_str(),
                qualified_name.as_str(),
                path.as_str(),
                signature.as_str(),
            )?;
            Some((raw_score, symbol))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left_symbol), (right_score, right_symbol)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_symbol.name.cmp(&right_symbol.name))
            .then_with(|| left_symbol.qualified_name.cmp(&right_symbol.qualified_name))
            .then_with(|| left_symbol.path.cmp(&right_symbol.path))
    });

    matches
        .into_iter()
        .map(|(raw_score, symbol)| RankedSearchRecord {
            item: symbol,
            score: normalized_rank_score(raw_score, SYMBOL_SEARCH_BUCKETS),
        })
        .collect()
}

pub(super) fn indexed_symbol_prefix_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, SymbolRecord>,
    query: &str,
    normalized_query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<SymbolRecord>> {
    let mut matches = index
        .search_prefix_hits(query, limit)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let symbol = lookup.get(hit.id.as_str())?.clone();
            let name = symbol.name.to_ascii_lowercase();
            let qualified_name = symbol.qualified_name.to_ascii_lowercase();
            let path = symbol.path.to_ascii_lowercase();
            let signature = symbol
                .signature
                .as_deref()
                .map(str::to_ascii_lowercase)
                .unwrap_or_default();
            let raw_score = symbol_match_score(
                normalized_query,
                name.as_str(),
                qualified_name.as_str(),
                path.as_str(),
                signature.as_str(),
            )?;
            Some((raw_score, symbol))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left_symbol), (right_score, right_symbol)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_symbol.name.cmp(&right_symbol.name))
            .then_with(|| left_symbol.qualified_name.cmp(&right_symbol.qualified_name))
            .then_with(|| left_symbol.path.cmp(&right_symbol.path))
    });

    matches
        .into_iter()
        .map(|(raw_score, symbol)| RankedSearchRecord {
            item: symbol,
            score: normalized_rank_score(raw_score, SYMBOL_SEARCH_BUCKETS),
        })
        .collect()
}

pub(super) fn indexed_example_exact_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, ExampleRecord>,
    metadata_lookup: &BTreeMap<String, ExampleSearchMetadata>,
    query: &str,
    normalized_query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<ExampleRecord>> {
    let mut matches = index
        .search_exact_hits(query, limit)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let example = lookup.get(hit.id.as_str())?.clone();
            let metadata = metadata_lookup
                .get(example.example_id.as_str())
                .cloned()
                .unwrap_or_default();
            let raw_score = raw_example_match_score(normalized_query, &example, &metadata)?;
            Some((raw_score, example))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left_example), (right_score, right_example)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_example.title.cmp(&right_example.title))
            .then_with(|| left_example.path.cmp(&right_example.path))
    });

    matches
        .into_iter()
        .map(|(raw_score, example)| RankedSearchRecord {
            item: example,
            score: normalized_rank_score(raw_score, EXAMPLE_SEARCH_BUCKETS),
        })
        .collect()
}

pub(super) fn indexed_example_prefix_matches(
    index: &SearchDocumentIndex,
    lookup: &BTreeMap<String, ExampleRecord>,
    metadata_lookup: &BTreeMap<String, ExampleSearchMetadata>,
    query: &str,
    normalized_query: &str,
    limit: usize,
) -> Vec<RankedSearchRecord<ExampleRecord>> {
    let mut matches = index
        .search_prefix_hits(query, limit)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|hit| {
            let example = lookup.get(hit.id.as_str())?.clone();
            let metadata = metadata_lookup
                .get(example.example_id.as_str())
                .cloned()
                .unwrap_or_default();
            let raw_score = raw_example_match_score(normalized_query, &example, &metadata)?;
            Some((raw_score, example))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left_example), (right_score, right_example)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_example.title.cmp(&right_example.title))
            .then_with(|| left_example.path.cmp(&right_example.path))
    });

    matches
        .into_iter()
        .map(|(raw_score, example)| RankedSearchRecord {
            item: example,
            score: normalized_rank_score(raw_score, EXAMPLE_SEARCH_BUCKETS),
        })
        .collect()
}
