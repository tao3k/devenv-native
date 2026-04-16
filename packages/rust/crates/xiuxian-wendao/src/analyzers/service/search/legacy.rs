use std::collections::BTreeMap;

use crate::analyzers::{ExampleRecord, ModuleRecord, SymbolRecord};

use super::super::helpers::{module_match_score, normalized_rank_score, symbol_match_score};
use super::documents::{ExampleSearchMetadata, raw_example_match_score};
use super::ranking::{
    EXAMPLE_SEARCH_BUCKETS, MODULE_SEARCH_BUCKETS, RankedSearchRecord, SYMBOL_SEARCH_BUCKETS,
};

pub(super) fn legacy_module_matches(
    normalized_query: &str,
    modules: &[ModuleRecord],
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    let mut matches = modules
        .iter()
        .filter_map(|module| {
            let qualified_name = module.qualified_name.to_ascii_lowercase();
            let path = module.path.to_ascii_lowercase();
            let score =
                module_match_score(normalized_query, qualified_name.as_str(), path.as_str())?;
            Some((score, module.clone()))
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

pub(super) fn legacy_symbol_matches(
    normalized_query: &str,
    symbols: &[SymbolRecord],
) -> Vec<RankedSearchRecord<SymbolRecord>> {
    let mut matches = symbols
        .iter()
        .filter_map(|symbol| {
            let name = symbol.name.to_ascii_lowercase();
            let qualified_name = symbol.qualified_name.to_ascii_lowercase();
            let path = symbol.path.to_ascii_lowercase();
            let signature = symbol
                .signature
                .as_deref()
                .map(str::to_ascii_lowercase)
                .unwrap_or_default();
            let score = symbol_match_score(
                normalized_query,
                name.as_str(),
                qualified_name.as_str(),
                path.as_str(),
                signature.as_str(),
            )?;
            Some((score, symbol.clone()))
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

pub(super) fn legacy_example_matches(
    normalized_query: &str,
    examples: &[ExampleRecord],
    metadata_lookup: &BTreeMap<String, ExampleSearchMetadata>,
) -> Vec<RankedSearchRecord<ExampleRecord>> {
    let mut matches = examples
        .iter()
        .filter_map(|example| {
            let metadata = metadata_lookup
                .get(example.example_id.as_str())
                .cloned()
                .unwrap_or_default();
            let score = raw_example_match_score(normalized_query, example, &metadata)?;
            Some((score, example.clone()))
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
