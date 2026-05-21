//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::{BTreeMap, BTreeSet};

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::build_projected_pages;

pub(crate) fn projection_page_lookup(
    analysis: &RepositoryAnalysisOutput,
) -> BTreeMap<String, Vec<String>> {
    let mut lookup: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for page in build_projected_pages(analysis) {
        for anchor in page
            .module_ids
            .iter()
            .chain(page.symbol_ids.iter())
            .chain(page.example_ids.iter())
            .chain(page.doc_ids.iter())
        {
            lookup
                .entry(anchor.clone())
                .or_default()
                .insert(page.page_id.clone());
        }
    }

    lookup
        .into_iter()
        .map(|(anchor, page_ids)| (anchor, page_ids.into_iter().collect::<Vec<_>>()))
        .collect()
}

pub(crate) fn projection_pages_for(
    anchor_id: &str,
    lookup: &BTreeMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    lookup.get(anchor_id).and_then(|page_ids| {
        let filtered = page_ids
            .iter()
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        (!filtered.is_empty()).then_some(filtered)
    })
}
