use crate::link_graph::index::{
    LinkGraphIndex, LinkGraphMatchStrategy, LinkGraphSearchOptions, normalize_path_filter,
    normalize_with_case, tokenize,
};
use regex::{Regex, RegexBuilder};

#[derive(Debug, Clone)]
pub struct SearchExecutionContext {
    pub bounded: usize,
    pub case_sensitive: bool,
    pub raw_query: String,
    pub clean_query: String,
    pub query_tokens: Vec<String>,
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub tag_all: Vec<String>,
    pub tag_any: Vec<String>,
    pub tag_not: Vec<String>,
    pub mention_filters: Vec<String>,
    pub regex: Option<Regex>,
}

impl LinkGraphIndex {
    pub(super) fn prepare_execution_context(
        query: &str,
        limit: usize,
        options: &LinkGraphSearchOptions,
    ) -> Option<SearchExecutionContext> {
        let raw_query = query.trim().to_string();
        let bounded = limit.max(1);
        let case_sensitive = options.case_sensitive;
        let clean_query = normalize_with_case(&raw_query, case_sensitive);
        let query_tokens = tokenize(&raw_query, case_sensitive);

        let include_paths: Vec<String> = options
            .filters
            .include_paths
            .iter()
            .map(|path| normalize_path_filter(path))
            .filter(|path| !path.is_empty())
            .collect();
        let exclude_paths: Vec<String> = options
            .filters
            .exclude_paths
            .iter()
            .map(|path| normalize_path_filter(path))
            .filter(|path| !path.is_empty())
            .collect();

        let (tag_all, tag_any, tag_not) = if let Some(tags) = options.filters.tags.as_ref() {
            (
                tags.all
                    .iter()
                    .map(|tag| normalize_with_case(tag, case_sensitive))
                    .collect::<Vec<String>>(),
                tags.any
                    .iter()
                    .map(|tag| normalize_with_case(tag, case_sensitive))
                    .collect::<Vec<String>>(),
                tags.not_tags
                    .iter()
                    .map(|tag| normalize_with_case(tag, case_sensitive))
                    .collect::<Vec<String>>(),
            )
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        let mention_filters: Vec<String> = options
            .filters
            .mentions_of
            .iter()
            .map(|phrase| normalize_with_case(phrase, case_sensitive))
            .filter(|phrase| !phrase.is_empty())
            .collect();

        let regex = if matches!(options.match_strategy, LinkGraphMatchStrategy::Re) {
            RegexBuilder::new(&raw_query)
                .case_insensitive(!case_sensitive)
                .build()
                .ok()
        } else {
            None
        };
        if matches!(options.match_strategy, LinkGraphMatchStrategy::Re) && regex.is_none() {
            return None;
        }

        Some(SearchExecutionContext {
            bounded,
            case_sensitive,
            raw_query,
            clean_query,
            query_tokens,
            include_paths,
            exclude_paths,
            tag_all,
            tag_any,
            tag_not,
            mention_filters,
            regex,
        })
    }

    pub(super) fn resolve_search_runtime_policy(
        options: &LinkGraphSearchOptions,
        _context: &SearchExecutionContext,
    ) -> SearchRuntimePolicy {
        let scope = LinkGraphIndex::effective_scope(&options.filters);
        let structural_edges_enabled = LinkGraphIndex::allows_structural_edges(&options.filters);
        let semantic_edges_enabled = LinkGraphIndex::allows_semantic_edges(&options.filters);
        let collapse_to_doc = options.filters.collapse_to_doc.unwrap_or(true);
        let per_doc_section_cap =
            LinkGraphIndex::effective_per_doc_section_cap(&options.filters, scope);
        let min_section_words =
            LinkGraphIndex::effective_min_section_words(&options.filters, scope);
        let max_heading_level = LinkGraphIndex::effective_max_heading_level(&options.filters);
        let max_tree_hops = options.filters.max_tree_hops;

        SearchRuntimePolicy {
            scope,
            structural_edges_enabled,
            semantic_edges_enabled,
            collapse_to_doc,
            per_doc_section_cap,
            min_section_words,
            max_heading_level,
            max_tree_hops,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchRuntimePolicy {
    pub scope: LinkGraphScope,
    pub structural_edges_enabled: bool,
    pub semantic_edges_enabled: bool,
    pub collapse_to_doc: bool,
    pub per_doc_section_cap: usize,
    pub min_section_words: usize,
    pub max_heading_level: usize,
    pub max_tree_hops: Option<usize>,
}

use crate::link_graph::LinkGraphScope;
