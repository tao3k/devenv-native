use crate::search::tantivy::document::{SearchDocument, SearchDocumentHit};
use crate::search::tantivy::index::core::SearchDocumentIndex;
use crate::search::tantivy::index::helpers::{
    candidate_limit, collect_hits, normalize_exact_query,
};
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::IndexRecordOption;
use tantivy::{DocAddress, TantivyDocument, TantivyError, Term};

impl SearchDocumentIndex {
    /// Run an exact query over the shared search fields.
    ///
    /// # Errors
    ///
    /// Returns an error when Tantivy cannot parse or execute the query.
    pub fn search_exact(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchDocument>, TantivyError> {
        let query = query.trim();
        if query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let searcher = self.reader.searcher();
        let query = self.build_exact_query(query)?;
        let top_docs = searcher.search(&*query, &TopDocs::with_limit(candidate_limit(limit)))?;
        self.collect_exact_documents(top_docs, limit)
    }

    /// Run an exact query and return lightweight hit metadata for caller-side rehydration.
    ///
    /// # Errors
    ///
    /// Returns an error when Tantivy cannot parse or execute the query.
    pub fn search_exact_hits(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchDocumentHit>, TantivyError> {
        let query = query.trim();
        if query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let searcher = self.reader.searcher();
        let query_object = self.build_exact_query(query)?;
        let top_docs =
            searcher.search(&*query_object, &TopDocs::with_limit(candidate_limit(limit)))?;
        collect_hits(&self.fields, &searcher, top_docs, limit)
    }

    fn build_exact_query(&self, query: &str) -> Result<Box<dyn Query>, TantivyError> {
        let normalized_query = normalize_exact_query(query);
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        for spec in self.fields.match_field_specs() {
            if let Some(exact_field) = spec.exact_field {
                let term = Term::from_field_text(exact_field, normalized_query.as_str());
                clauses.push((
                    Occur::Should,
                    Box::new(BoostQuery::new(
                        Box::new(TermQuery::new(term, IndexRecordOption::Basic)),
                        spec.query_boost,
                    )),
                ));
            }
        }

        let parser = QueryParser::for_index(&self.index, self.fields.text_fields());
        clauses.push((Occur::Should, parser.parse_query(query)?));
        Ok(Box::new(BooleanQuery::new(clauses)))
    }

    fn collect_exact_documents(
        &self,
        top_docs: Vec<(f32, DocAddress)>,
        limit: usize,
    ) -> Result<Vec<SearchDocument>, TantivyError> {
        let searcher = self.reader.searcher();
        let mut seen_ids = std::collections::HashSet::new();
        top_docs
            .into_iter()
            .map(|(_, doc_address)| {
                let document: TantivyDocument = searcher.doc(doc_address)?;
                Ok(self.fields.parse_document(&document))
            })
            .filter_map(|record| match record {
                Ok(record) if seen_ids.insert(record.id.clone()) => Some(Ok(record)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .take(limit)
            .collect()
    }
}
