use crate::search::fuzzy::{FuzzyMatch, FuzzySearchOptions};
use crate::search::tantivy::document::SearchDocumentHit;
use crate::search::tantivy::index::core::SearchDocumentIndex;
use crate::search::tantivy::matcher::{TantivyDocumentMatch, TantivyMatcher};
use tantivy::TantivyError;

impl SearchDocumentIndex {
    /// Run a fuzzy query over the shared search fields.
    ///
    /// # Errors
    ///
    /// Returns an error when Tantivy cannot execute the fuzzy query.
    pub fn search_fuzzy(
        &self,
        query: &str,
        limit: usize,
        options: FuzzySearchOptions,
    ) -> Result<Vec<FuzzyMatch<crate::search::tantivy::document::SearchDocument>>, TantivyError>
    {
        let query = query.trim();
        if query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let matcher = TantivyMatcher::new(
            &self.index,
            self.fields.text_fields(),
            self.fields.match_field_specs().to_vec(),
            options,
        );
        let raw_matches = matcher.search_with_fields(query, limit)?;

        let mut records = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        for raw_match in raw_matches {
            let record = self.fields.parse_document(&raw_match.item);
            if !seen_ids.insert(record.id.clone()) {
                continue;
            }
            records.push(FuzzyMatch {
                item: record,
                matched_text: raw_match.matched_text,
                score: raw_match.score,
                distance: raw_match.distance,
            });
            if records.len() >= limit {
                break;
            }
        }

        Ok(records)
    }

    /// Run a fuzzy query and return lightweight hit metadata for caller-side rehydration.
    ///
    /// # Errors
    ///
    /// Returns an error when Tantivy cannot execute the fuzzy query.
    pub fn search_fuzzy_hits(
        &self,
        query: &str,
        limit: usize,
        options: FuzzySearchOptions,
    ) -> Result<Vec<SearchDocumentHit>, TantivyError> {
        let query = query.trim();
        if query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let matcher = TantivyMatcher::new(
            &self.index,
            self.fields.text_fields(),
            self.fields.match_field_specs().to_vec(),
            options,
        );
        let raw_matches = matcher.search_with_fields(query, limit)?;
        Ok(self.collect_fuzzy_hits(raw_matches, limit))
    }

    fn collect_fuzzy_hits(
        &self,
        raw_matches: Vec<TantivyDocumentMatch>,
        limit: usize,
    ) -> Vec<SearchDocumentHit> {
        let mut seen_ids = std::collections::HashSet::new();
        raw_matches
            .into_iter()
            .map(|raw_match| {
                self.fields.parse_hit(
                    &raw_match.item,
                    raw_match.score,
                    raw_match.matched_field,
                    Some(raw_match.matched_text),
                    raw_match.distance,
                )
            })
            .filter(|hit| seen_ids.insert(hit.id.clone()))
            .take(limit)
            .collect()
    }
}
