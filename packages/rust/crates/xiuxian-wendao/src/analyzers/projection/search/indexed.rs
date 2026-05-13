//! `analyzers::projection::search::indexed` owns Wendao projection search indexed behavior.

use std::collections::HashMap;

use crate::analyzers::{ProjectedPageRecord, ProjectionPageKind};
use crate::search::{FuzzySearchOptions, SearchDocument, SearchDocumentIndex};

use super::mapping::{map_fuzzy_search_documents_to_pages, map_search_documents_to_pages};

pub(super) fn search_indexed_projected_pages(
    query: &str,
    kind_filter: Option<ProjectionPageKind>,
    pages: &[ProjectedPageRecord],
    limit: usize,
    options: FuzzySearchOptions,
) -> Option<Vec<(u8, ProjectedPageRecord)>> {
    let (search_index, page_by_id) = build_projected_page_search_index(pages).ok()?;
    Some(search_projected_pages_with_index(
        query,
        kind_filter,
        &search_index,
        &page_by_id,
        limit,
        options,
    ))
}

pub(crate) fn search_projected_pages_with_index(
    query: &str,
    kind_filter: Option<ProjectionPageKind>,
    search_index: &SearchDocumentIndex,
    page_by_id: &HashMap<String, ProjectedPageRecord>,
    limit: usize,
    options: FuzzySearchOptions,
) -> Vec<(u8, ProjectedPageRecord)> {
    let normalized_query = query.to_ascii_lowercase();

    let exact_matches = search_index
        .search_exact_hits(query, limit.saturating_mul(2))
        .ok()
        .map(|records| {
            map_search_documents_to_pages(
                records,
                page_by_id,
                kind_filter,
                limit,
                normalized_query.as_str(),
                Some(50),
            )
        })
        .unwrap_or_default();
    if !exact_matches.is_empty() {
        return exact_matches;
    }

    let prefix_matches = search_index
        .search_prefix_hits(query, limit.saturating_mul(2))
        .ok()
        .map(|records| {
            map_search_documents_to_pages(
                records,
                page_by_id,
                kind_filter,
                limit,
                normalized_query.as_str(),
                Some(55),
            )
        })
        .unwrap_or_default();
    if !prefix_matches.is_empty() {
        return prefix_matches;
    }

    let fuzzy_matches = search_index
        .search_fuzzy_hits(query, limit.saturating_mul(2), options)
        .ok()
        .map(|records| {
            map_fuzzy_search_documents_to_pages(
                records,
                page_by_id,
                kind_filter,
                limit,
                normalized_query.as_str(),
            )
        })
        .unwrap_or_default();
    if !fuzzy_matches.is_empty() {
        return fuzzy_matches;
    }

    Vec::new()
}
/// `build_projected_page_search_index` public function boundary for Wendao.
pub fn build_projected_page_search_index(
    pages: &[ProjectedPageRecord],
) -> Result<(SearchDocumentIndex, HashMap<String, ProjectedPageRecord>), String> {
    let search_index = SearchDocumentIndex::new();
    let mut page_by_id = HashMap::new();
    let mut documents = Vec::new();

    for page in pages {
        page_by_id.insert(page.page_id.clone(), page.clone());
        documents.push(projected_page_search_document(page));
    }

    search_index
        .add_documents(documents)
        .map_err(|error| error.to_string())?;

    Ok((search_index, page_by_id))
}

fn projected_page_search_document(page: &ProjectedPageRecord) -> SearchDocument {
    let mut terms = page.keywords.clone();
    terms.extend(page.doc_ids.iter().cloned());
    terms.extend(page.paths.iter().cloned());
    terms.extend(page.module_ids.iter().cloned());
    terms.extend(page.symbol_ids.iter().cloned());
    terms.extend(page.example_ids.iter().cloned());
    terms.extend(page.format_hints.iter().cloned());
    terms.push(page.doc_id.clone());
    terms.push(projection_kind_token(page.kind).to_string());
    terms.sort();
    terms.dedup();

    SearchDocument {
        id: page.page_id.clone(),
        title: page.title.clone(),
        kind: projection_kind_token(page.kind).to_string(),
        path: page.path.clone(),
        scope: page.repo_id.clone(),
        namespace: page.doc_id.clone(),
        terms,
    }
}

fn projection_kind_token(kind: ProjectionPageKind) -> &'static str {
    match kind {
        ProjectionPageKind::Reference => "reference",
        ProjectionPageKind::HowTo => "howto",
        ProjectionPageKind::Tutorial => "tutorial",
        ProjectionPageKind::Explanation => "explanation",
    }
}
