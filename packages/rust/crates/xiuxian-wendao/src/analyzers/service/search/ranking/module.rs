#[cfg(feature = "repo-lexical-index")]
use std::collections::BTreeMap;
use std::collections::HashSet;

use crate::analyzers::ModuleRecord;
use crate::analyzers::service::normalized_rank_score;
#[cfg(feature = "repo-lexical-index")]
use crate::search::SearchDocumentIndex;
use crate::search::{FuzzyMatcher, FuzzySearchOptions, LexicalMatcher};

#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::documents::{
    build_search_document_index, module_search_document,
};
#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::indexed_exact::{
    indexed_module_exact_matches, indexed_module_prefix_matches,
};
#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::service::search::indexed_fuzzy::indexed_module_fuzzy_matches;
use crate::analyzers::service::search::legacy::legacy_module_matches;

use super::{MODULE_SEARCH_BUCKETS, RankedSearchRecord, search_candidate_limit};

fn module_qualified_name(module: &ModuleRecord) -> &str {
    module.qualified_name.as_str()
}

fn lexical_module_fuzzy_matches(
    query: &str,
    modules: &[ModuleRecord],
    limit: usize,
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    let matcher = LexicalMatcher::new(
        modules,
        module_qualified_name,
        FuzzySearchOptions::camel_case_symbol(),
    );
    matcher
        .search(query, limit)
        .unwrap_or_default()
        .into_iter()
        .map(|matched_module| RankedSearchRecord {
            item: matched_module.item,
            score: f64::from(matched_module.score),
        })
        .collect()
}

pub(crate) fn ranked_module_matches(
    query: &str,
    modules: &[ModuleRecord],
    limit: usize,
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    #[cfg(feature = "repo-lexical-index")]
    {
        let lookup = modules
            .iter()
            .map(|module| (module.module_id.clone(), module.clone()))
            .collect::<BTreeMap<_, _>>();
        let Some(index) = build_search_document_index(modules.iter().map(module_search_document))
        else {
            return ranked_module_matches_without_index(query, modules, limit);
        };
        ranked_module_matches_from_index(query, modules, &lookup, &index, limit)
    }

    #[cfg(not(feature = "repo-lexical-index"))]
    {
        ranked_module_matches_without_index(query, modules, limit)
    }
}

#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub fn ranked_module_matches_with_artifacts(
    query: &str,
    modules: &[ModuleRecord],
    lookup: &BTreeMap<String, ModuleRecord>,
    index: &SearchDocumentIndex,
    limit: usize,
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    ranked_module_matches_from_index(query, modules, lookup, index, limit)
}

#[cfg(feature = "repo-lexical-index")]
fn ranked_module_matches_from_index(
    query: &str,
    modules: &[ModuleRecord],
    lookup: &BTreeMap<String, ModuleRecord>,
    index: &SearchDocumentIndex,
    limit: usize,
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    if modules.is_empty() || limit == 0 {
        return Vec::new();
    }

    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return modules
            .iter()
            .take(limit)
            .cloned()
            .map(|module| RankedSearchRecord {
                item: module,
                score: normalized_rank_score(0, MODULE_SEARCH_BUCKETS),
            })
            .collect();
    }

    let search_limit = search_candidate_limit(limit);
    let mut ranked = Vec::new();
    let mut seen_ids = HashSet::new();

    for candidate in indexed_module_exact_matches(
        index,
        lookup,
        query,
        normalized_query.as_str(),
        search_limit,
    ) {
        if seen_ids.insert(candidate.item.module_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in indexed_module_prefix_matches(
        index,
        lookup,
        query,
        normalized_query.as_str(),
        search_limit,
    ) {
        if seen_ids.insert(candidate.item.module_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in indexed_module_fuzzy_matches(index, lookup, query, search_limit) {
        if seen_ids.insert(candidate.item.module_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in lexical_module_fuzzy_matches(query, modules, search_limit) {
        if seen_ids.insert(candidate.item.module_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }

    for candidate in legacy_module_matches(normalized_query.as_str(), modules) {
        if seen_ids.insert(candidate.item.module_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                break;
            }
        }
    }

    ranked
}

fn ranked_module_matches_without_index(
    query: &str,
    modules: &[ModuleRecord],
    limit: usize,
) -> Vec<RankedSearchRecord<ModuleRecord>> {
    if modules.is_empty() || limit == 0 {
        return Vec::new();
    }

    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return modules
            .iter()
            .take(limit)
            .cloned()
            .map(|module| RankedSearchRecord {
                item: module,
                score: normalized_rank_score(0, MODULE_SEARCH_BUCKETS),
            })
            .collect();
    }

    let search_limit = search_candidate_limit(limit);
    let mut ranked = Vec::new();
    let mut seen_ids = HashSet::new();
    for candidate in lexical_module_fuzzy_matches(query, modules, search_limit) {
        if seen_ids.insert(candidate.item.module_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                return ranked;
            }
        }
    }
    for candidate in legacy_module_matches(normalized_query.as_str(), modules) {
        if seen_ids.insert(candidate.item.module_id.clone()) {
            ranked.push(candidate);
            if ranked.len() >= limit {
                break;
            }
        }
    }
    ranked
}
