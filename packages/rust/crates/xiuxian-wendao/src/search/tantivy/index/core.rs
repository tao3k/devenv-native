//! `search::tantivy::index::core` owns Wendao tantivy index core behavior.

use crate::search::tantivy::fields::SearchDocumentFields;
use crate::search::tantivy::tokenizer::register_search_tokenizer;
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyError};

use crate::search::tantivy::document::SearchDocument;

/// Shared Tantivy-backed index for domain-agnostic search documents.
#[derive(Clone)]
pub struct SearchDocumentIndex {
    /// In-memory Tantivy index.
    pub index: Index,
    /// Shared document fields.
    pub fields: SearchDocumentFields,
    pub(crate) reader: IndexReader,
}

impl SearchDocumentIndex {
    /// Create an empty in-memory search index using the shared schema.
    ///
    /// # Panics
    ///
    /// Panics when Tantivy cannot initialize the shared in-memory reader.
    #[must_use]
    pub fn new() -> Self {
        let fields = SearchDocumentFields::new();
        let index = Index::create_in_ram(fields.schema.clone());
        register_search_tokenizer(&index);
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .unwrap_or_else(|error| panic!("shared Tantivy reader should initialize: {error}"));
        Self {
            index,
            fields,
            reader,
        }
    }

    /// Add a single shared search document to the index.
    ///
    /// # Errors
    ///
    /// Returns an error when Tantivy cannot write or commit the document.
    pub fn add_document(&self, document: &SearchDocument) -> Result<(), TantivyError> {
        self.add_documents(std::iter::once(document.clone()))
    }

    /// Add multiple shared search documents in one writer commit.
    ///
    /// # Errors
    ///
    /// Returns an error when Tantivy cannot open a writer or commit the batch.
    pub fn add_documents<I>(&self, documents: I) -> Result<(), TantivyError>
    where
        I: IntoIterator<Item = SearchDocument>,
    {
        let mut writer = self.index.writer(50_000_000)?;
        for document in documents {
            let _ = writer.add_document(self.fields.make_document(&document));
        }
        let _ = writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }
}

impl Default for SearchDocumentIndex {
    fn default() -> Self {
        Self::new()
    }
}
