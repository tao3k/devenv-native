//! `search::tantivy::fields` owns Wendao search tantivy fields behavior.

use std::collections::HashSet;

use tantivy::TantivyDocument;
use tantivy::schema::{
    Field, IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions, Value,
};

use super::document::{SearchDocument, SearchDocumentHit, SearchDocumentMatchField};

pub(crate) const SEARCH_CODE_TOKENIZER: &str = "search_code_tokenizer";

#[derive(Debug, Clone, Copy)]
pub(crate) struct SearchFieldSpec {
    pub(crate) label: SearchDocumentMatchField,
    pub(crate) text_field: Field,
    pub(crate) exact_field: Option<Field>,
    pub(crate) query_boost: f32,
    pub(crate) fuzzy_boost: f32,
}

/// Shared field set for common Tantivy-backed search documents.
#[derive(Debug, Clone)]
pub struct SearchDocumentFields {
    /// Tantivy schema built from the shared field set.
    pub schema: tantivy::schema::Schema,
    /// Stable record identifier.
    pub id: Field,
    /// Case-folded exact title field.
    pub title_exact: Field,
    /// Tokenized title field.
    pub title_text: Field,
    /// Domain-specific kind field.
    pub kind: Field,
    /// Case-folded exact path field.
    pub path_exact: Field,
    /// Tokenized path field.
    pub path_text: Field,
    /// Coarse scope field such as repo or source.
    pub scope: Field,
    /// Case-folded exact namespace field.
    pub namespace_exact: Field,
    /// Tokenized namespace field.
    pub namespace_text: Field,
    /// Additional queryable terms.
    pub terms_text: Field,
}

impl SearchDocumentFields {
    /// Build the shared search schema.
    #[must_use]
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();
        let text_options = code_text_options();
        let id = schema_builder.add_text_field("id", STRING | STORED);
        let title_exact = schema_builder.add_text_field("title_exact", STRING);
        let title_text = schema_builder.add_text_field("title_text", text_options.clone());
        let kind = schema_builder.add_text_field("kind", STRING | STORED);
        let path_exact = schema_builder.add_text_field("path_exact", STRING);
        let path_text = schema_builder.add_text_field("path_text", text_options.clone());
        let scope = schema_builder.add_text_field("scope", STRING | STORED);
        let namespace_exact = schema_builder.add_text_field("namespace_exact", STRING);
        let namespace_text = schema_builder.add_text_field("namespace_text", text_options.clone());
        let terms_text = schema_builder.add_text_field("terms_text", text_options);

        Self {
            schema: schema_builder.build(),
            id,
            title_exact,
            title_text,
            kind,
            path_exact,
            path_text,
            scope,
            namespace_exact,
            namespace_text,
            terms_text,
        }
    }

    /// Default fields used for exact lookup.
    #[must_use]
    pub fn default_fields(&self) -> Vec<Field> {
        self.text_fields()
    }

    /// Text fields used for parser-driven exact and fuzzy lookup.
    #[must_use]
    pub fn text_fields(&self) -> Vec<Field> {
        self.match_field_specs()
            .iter()
            .map(|spec| spec.text_field)
            .collect()
    }

    #[must_use]
    pub(crate) fn match_field_specs(&self) -> [SearchFieldSpec; 4] {
        [
            SearchFieldSpec {
                label: SearchDocumentMatchField::Title,
                text_field: self.title_text,
                exact_field: Some(self.title_exact),
                query_boost: 5.0,
                fuzzy_boost: 1.0,
            },
            SearchFieldSpec {
                label: SearchDocumentMatchField::Namespace,
                text_field: self.namespace_text,
                exact_field: Some(self.namespace_exact),
                query_boost: 3.5,
                fuzzy_boost: 0.92,
            },
            SearchFieldSpec {
                label: SearchDocumentMatchField::Path,
                text_field: self.path_text,
                exact_field: Some(self.path_exact),
                query_boost: 3.0,
                fuzzy_boost: 0.88,
            },
            SearchFieldSpec {
                label: SearchDocumentMatchField::Terms,
                text_field: self.terms_text,
                exact_field: None,
                query_boost: 2.5,
                fuzzy_boost: 0.84,
            },
        ]
    }

    /// Build one Tantivy document from a shared record.
    #[must_use]
    pub fn make_document(&self, record: &SearchDocument) -> TantivyDocument {
        let mut document = TantivyDocument::default();
        document.add_text(self.id, &record.id);
        document.add_text(self.title_exact, normalize_exact_value(&record.title));
        document.add_text(self.title_text, &record.title);
        document.add_text(self.kind, &record.kind);
        document.add_text(self.path_exact, normalize_exact_value(&record.path));
        document.add_text(self.path_text, &record.path);
        document.add_text(self.scope, &record.scope);
        document.add_text(
            self.namespace_exact,
            normalize_exact_value(&record.namespace),
        );
        document.add_text(self.namespace_text, &record.namespace);

        let mut seen_terms = HashSet::new();
        for term in &record.terms {
            let normalized = term.trim();
            if normalized.is_empty() || !seen_terms.insert(normalized.to_ascii_lowercase()) {
                continue;
            }
            document.add_text(self.terms_text, normalized);
        }

        document
    }

    /// Parse one shared record from a Tantivy document.
    #[must_use]
    pub fn parse_document(&self, document: &TantivyDocument) -> SearchDocument {
        let mut terms = document
            .get_all(self.terms_text)
            .filter_map(|value| value.as_str())
            .map(str::to_string)
            .collect::<Vec<_>>();
        terms.sort();
        terms.dedup();

        SearchDocument {
            id: field_string(document, self.id),
            title: field_string(document, self.title_text),
            kind: field_string(document, self.kind),
            path: field_string(document, self.path_text),
            scope: field_string(document, self.scope),
            namespace: field_string(document, self.namespace_text),
            terms,
        }
    }

    #[must_use]
    pub(crate) fn parse_hit(
        &self,
        document: &TantivyDocument,
        score: f32,
        matched_field: Option<SearchDocumentMatchField>,
        matched_text: Option<String>,
        distance: usize,
    ) -> SearchDocumentHit {
        SearchDocumentHit {
            id: field_string(document, self.id),
            matched_field,
            matched_text,
            score,
            distance,
        }
    }
}

impl Default for SearchDocumentFields {
    fn default() -> Self {
        Self::new()
    }
}

fn field_string(document: &TantivyDocument, field: Field) -> String {
    document
        .get_first(field)
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string()
}

fn code_text_options() -> TextOptions {
    TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(SEARCH_CODE_TOKENIZER)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored()
}

fn normalize_exact_value(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).collect()
}
