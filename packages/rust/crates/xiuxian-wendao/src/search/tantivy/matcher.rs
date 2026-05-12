//! `search::tantivy::matcher` owns Wendao search tantivy matcher behavior.

use std::cmp::Ordering;

use crate::search::fuzzy::{FuzzyMatch, FuzzyMatcher, FuzzySearchOptions};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Value};
use tantivy::{DocAddress, Index, Searcher, TantivyDocument, TantivyError};

use super::compare::{best_match_candidate, collect_lowercase_chars};
use super::document::SearchDocumentMatchField;
use super::fields::SearchFieldSpec;

const FUZZY_CANDIDATE_WINDOW_CAP: usize = 96;
const FUZZY_CANDIDATE_WINDOW_MULTIPLIER: usize = 3;

/// One Tantivy-backed fuzzy match with matched-field metadata.
#[derive(Debug, Clone)]
pub struct TantivyDocumentMatch {
    /// Raw Tantivy document.
    pub item: TantivyDocument,
    /// Best-matching stored field when identified.
    pub matched_field: Option<SearchDocumentMatchField>,
    /// Best-matching text fragment.
    pub matched_text: String,
    /// Adjusted fuzzy score.
    pub score: f32,
    /// Edit distance for the chosen fragment.
    pub distance: usize,
}

/// Shared Tantivy-backed fuzzy matcher for text fields.
pub struct TantivyMatcher<'a> {
    index: &'a Index,
    default_fields: Vec<Field>,
    match_fields: Vec<SearchFieldSpec>,
    options: FuzzySearchOptions,
}

impl<'a> TantivyMatcher<'a> {
    /// Create a Tantivy fuzzy matcher for one primary match field.
    #[must_use]
    pub(crate) fn new(
        index: &'a Index,
        default_fields: Vec<Field>,
        match_fields: Vec<SearchFieldSpec>,
        options: FuzzySearchOptions,
    ) -> Self {
        Self {
            index,
            default_fields,
            match_fields,
            options,
        }
    }

    /// Search with fuzzy field metadata retained for rehydration-heavy callers.
    ///
    /// # Errors
    ///
    /// Returns an error when Tantivy cannot parse or execute the query.
    pub(crate) fn search_with_fields(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<TantivyDocumentMatch>, TantivyError> {
        let query = query.trim();
        if query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let mut scratch = TantivyMatchScratch::from_query(query);

        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let mut parser = QueryParser::for_index(self.index, self.default_fields.clone());
        for spec in &self.match_fields {
            parser.set_field_fuzzy(
                spec.text_field,
                false,
                self.options.max_distance.min(2),
                self.options.transposition,
            );
        }
        let query_object = parser.parse_query(query)?;
        let candidate_limit = limit
            .max(1)
            .saturating_mul(FUZZY_CANDIDATE_WINDOW_MULTIPLIER)
            .min(FUZZY_CANDIDATE_WINDOW_CAP);
        let top_docs = searcher.search(&query_object, &TopDocs::with_limit(candidate_limit))?;

        let mut matches = collect_tantivy_matches(self, &searcher, top_docs, query, &mut scratch)?;

        matches.sort_by(compare_tantivy_matches);
        matches.truncate(limit);
        Ok(matches)
    }
}

fn collect_tantivy_matches(
    matcher: &TantivyMatcher<'_>,
    searcher: &Searcher,
    top_docs: Vec<(f32, DocAddress)>,
    query: &str,
    scratch: &mut TantivyMatchScratch,
) -> Result<Vec<TantivyDocumentMatch>, TantivyError> {
    top_docs
        .into_iter()
        .filter_map(|(_score, doc_address)| {
            match_tantivy_document(matcher, searcher, doc_address, query, scratch).transpose()
        })
        .collect()
}

fn match_tantivy_document(
    matcher: &TantivyMatcher<'_>,
    searcher: &Searcher,
    doc_address: DocAddress,
    query: &str,
    scratch: &mut TantivyMatchScratch,
) -> Result<Option<TantivyDocumentMatch>, TantivyError> {
    let document: TantivyDocument = searcher.doc(doc_address)?;
    Ok(
        best_tantivy_document_match(matcher, &document, query, scratch).map(|best| {
            TantivyDocumentMatch {
                item: document,
                matched_field: best.field,
                matched_text: best.text,
                score: best.score,
                distance: best.distance,
            }
        }),
    )
}

fn best_tantivy_document_match(
    matcher: &TantivyMatcher<'_>,
    document: &TantivyDocument,
    query: &str,
    scratch: &mut TantivyMatchScratch,
) -> Option<TantivyBestMatch> {
    matcher
        .match_fields
        .iter()
        .filter_map(|spec| best_tantivy_field_match(matcher, document, query, spec, scratch))
        .fold(None, select_better_tantivy_match)
}

fn best_tantivy_field_match(
    matcher: &TantivyMatcher<'_>,
    document: &TantivyDocument,
    query: &str,
    spec: &SearchFieldSpec,
    scratch: &mut TantivyMatchScratch,
) -> Option<TantivyBestMatch> {
    document
        .get_all(spec.text_field)
        .filter_map(|value| value.as_str())
        .filter_map(|stored_text| matcher.match_stored_text(query, spec, stored_text, scratch))
        .fold(None, select_better_tantivy_match)
}

fn select_better_tantivy_match(
    best: Option<TantivyBestMatch>,
    candidate: TantivyBestMatch,
) -> Option<TantivyBestMatch> {
    match best {
        Some(current)
            if compare_tantivy_match_parts(candidate.as_parts(), current.as_parts()).is_ge() =>
        {
            Some(current)
        }
        _ => Some(candidate),
    }
}

impl TantivyMatcher<'_> {
    fn match_stored_text(
        &self,
        query: &str,
        spec: &SearchFieldSpec,
        stored_text: &str,
        scratch: &mut TantivyMatchScratch,
    ) -> Option<TantivyBestMatch> {
        best_match_candidate(
            query,
            scratch.query_chars.as_slice(),
            stored_text,
            self.options,
            &mut scratch.candidate_chars,
            &mut scratch.scratch,
            &mut scratch.seen_ranges,
            &mut scratch.boundary_scratch,
        )
        .map(|(matched_text, score)| TantivyBestMatch {
            field: Some(spec.label),
            text: matched_text,
            score: score.score * spec.fuzzy_boost,
            distance: score.distance,
        })
    }
}

struct TantivyMatchScratch {
    query_chars: Vec<char>,
    candidate_chars: Vec<char>,
    scratch: Vec<usize>,
    seen_ranges: Vec<(usize, usize)>,
    boundary_scratch: Vec<usize>,
}

impl TantivyMatchScratch {
    fn from_query(query: &str) -> Self {
        let mut query_chars = Vec::new();
        collect_lowercase_chars(query, &mut query_chars);
        Self {
            query_chars,
            candidate_chars: Vec::new(),
            scratch: Vec::new(),
            seen_ranges: Vec::new(),
            boundary_scratch: Vec::new(),
        }
    }
}

struct TantivyBestMatch {
    field: Option<SearchDocumentMatchField>,
    text: String,
    score: f32,
    distance: usize,
}

impl TantivyBestMatch {
    fn as_parts(&self) -> TantivyMatchParts<'_> {
        TantivyMatchParts {
            field: self.field,
            text: self.text.as_str(),
            score: self.score,
            distance: self.distance,
        }
    }
}

impl FuzzyMatcher<TantivyDocument> for TantivyMatcher<'_> {
    type Error = TantivyError;

    fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<FuzzyMatch<TantivyDocument>>, Self::Error> {
        let matches = self
            .search_with_fields(query, limit)?
            .into_iter()
            .map(|hit| FuzzyMatch {
                item: hit.item,
                matched_text: hit.matched_text,
                score: hit.score,
                distance: hit.distance,
            })
            .collect::<Vec<FuzzyMatch<TantivyDocument>>>();
        Ok(matches)
    }
}

fn compare_tantivy_matches(left: &TantivyDocumentMatch, right: &TantivyDocumentMatch) -> Ordering {
    compare_tantivy_match_parts(
        TantivyMatchParts::from_match(left),
        TantivyMatchParts::from_match(right),
    )
}

#[derive(Clone, Copy)]
struct TantivyMatchParts<'a> {
    field: Option<SearchDocumentMatchField>,
    text: &'a str,
    score: f32,
    distance: usize,
}

impl<'a> TantivyMatchParts<'a> {
    fn from_match(hit: &'a TantivyDocumentMatch) -> Self {
        Self {
            field: hit.matched_field,
            text: hit.matched_text.as_str(),
            score: hit.score,
            distance: hit.distance,
        }
    }
}

fn compare_tantivy_match_parts(
    left: TantivyMatchParts<'_>,
    right: TantivyMatchParts<'_>,
) -> Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.distance.cmp(&right.distance))
        .then_with(|| field_rank(left.field).cmp(&field_rank(right.field)))
        .then_with(|| left.text.len().cmp(&right.text.len()))
        .then_with(|| left.text.cmp(right.text))
}

fn field_rank(field: Option<SearchDocumentMatchField>) -> u8 {
    match field {
        Some(SearchDocumentMatchField::Title) => 0,
        Some(SearchDocumentMatchField::Namespace) => 1,
        Some(SearchDocumentMatchField::Path) => 2,
        Some(SearchDocumentMatchField::Terms) => 3,
        None => u8::MAX,
    }
}
