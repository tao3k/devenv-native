//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::search::tantivy::document::SearchDocumentHit;
use crate::search::tantivy::fields::SearchDocumentFields;
use std::collections::HashSet;
use tantivy::{DocAddress, Searcher, TantivyDocument, TantivyError};

const SEARCH_CANDIDATE_WINDOW_CAP: usize = 96;
const SEARCH_CANDIDATE_WINDOW_MULTIPLIER: usize = 3;

pub(crate) fn candidate_limit(limit: usize) -> usize {
    limit
        .max(1)
        .saturating_mul(SEARCH_CANDIDATE_WINDOW_MULTIPLIER)
        .min(SEARCH_CANDIDATE_WINDOW_CAP)
}

pub(crate) fn normalize_exact_query(query: &str) -> String {
    query.trim().chars().flat_map(char::to_lowercase).collect()
}

pub(crate) fn collect_hits(
    fields: &SearchDocumentFields,
    searcher: &Searcher,
    top_docs: Vec<(f32, DocAddress)>,
    limit: usize,
) -> Result<Vec<SearchDocumentHit>, TantivyError> {
    top_docs
        .into_iter()
        .try_fold(
            (Vec::new(), HashSet::new()),
            |(mut hits, mut seen_ids), (score, doc_address)| {
                if hits.len() >= limit {
                    return Ok((hits, seen_ids));
                }
                let document: TantivyDocument = searcher.doc(doc_address)?;
                let hit = fields.parse_hit(&document, score, None, None, 0);
                if seen_ids.insert(hit.id.clone()) {
                    hits.push(hit);
                }
                Ok((hits, seen_ids))
            },
        )
        .map(|(hits, _)| hits)
}
