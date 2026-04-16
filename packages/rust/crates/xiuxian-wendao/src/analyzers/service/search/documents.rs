use crate::analyzers::ExampleRecord;
use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(feature = "repo-lexical-index")]
use crate::analyzers::{ModuleRecord, SymbolRecord};
#[cfg(feature = "repo-lexical-index")]
use crate::search::{SearchDocument, SearchDocumentIndex};
use std::collections::BTreeMap;

use super::super::helpers::{
    example_match_score, example_relation_lookup, related_modules_for_example,
    related_symbols_for_example,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct ExampleSearchMetadata {
    pub(super) related_symbols: Vec<String>,
    pub(super) related_modules: Vec<String>,
}

#[cfg(feature = "repo-lexical-index")]
pub(super) fn build_search_document_index<I>(documents: I) -> Option<SearchDocumentIndex>
where
    I: IntoIterator<Item = SearchDocument>,
{
    let index = SearchDocumentIndex::new();
    index.add_documents(documents).ok()?;
    Some(index)
}

#[cfg(feature = "repo-lexical-index")]
pub(super) fn module_search_document(module: &ModuleRecord) -> SearchDocument {
    let namespace = module
        .qualified_name
        .rsplit_once('.')
        .map(|(namespace, _name)| namespace.to_string())
        .unwrap_or_default();

    SearchDocument {
        id: module.module_id.clone(),
        title: module.qualified_name.clone(),
        kind: "module".to_string(),
        path: module.path.clone(),
        scope: module.repo_id.clone(),
        namespace,
        terms: vec![module.qualified_name.clone(), module.path.clone()],
    }
}

#[cfg(feature = "repo-lexical-index")]
pub(super) fn symbol_search_document(symbol: &SymbolRecord) -> SearchDocument {
    let mut terms = vec![
        symbol.name.clone(),
        symbol.qualified_name.clone(),
        symbol.path.clone(),
    ];
    if let Some(signature) = &symbol.signature {
        terms.push(signature.clone());
    }
    if let Some(module_id) = &symbol.module_id {
        terms.push(module_id.clone());
    }
    terms.extend(symbol.attributes.values().cloned());

    SearchDocument {
        id: symbol.symbol_id.clone(),
        title: symbol.qualified_name.clone(),
        kind: format!("{:?}", symbol.kind).to_ascii_lowercase(),
        path: symbol.path.clone(),
        scope: symbol.repo_id.clone(),
        namespace: symbol.module_id.clone().unwrap_or_default(),
        terms,
    }
}

#[cfg(feature = "repo-lexical-index")]
pub(super) fn example_search_document(
    example: &ExampleRecord,
    metadata: &ExampleSearchMetadata,
) -> SearchDocument {
    let title = std::iter::once(example.title.clone())
        .chain(metadata.related_symbols.iter().cloned())
        .chain(metadata.related_modules.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ");

    let mut terms = vec![example.title.clone(), example.path.clone()];
    if let Some(summary) = &example.summary {
        terms.push(summary.clone());
    }
    terms.extend(metadata.related_symbols.iter().cloned());
    terms.extend(metadata.related_modules.iter().cloned());

    SearchDocument {
        id: example.example_id.clone(),
        title,
        kind: "example".to_string(),
        path: example.path.clone(),
        scope: example.repo_id.clone(),
        namespace: metadata
            .related_modules
            .first()
            .cloned()
            .unwrap_or_default(),
        terms,
    }
}

pub(super) fn build_example_metadata_lookup(
    analysis: &RepositoryAnalysisOutput,
) -> BTreeMap<String, ExampleSearchMetadata> {
    let relation_lookup = example_relation_lookup(&analysis.relations);
    analysis
        .examples
        .iter()
        .map(|example| {
            (
                example.example_id.clone(),
                ExampleSearchMetadata {
                    related_symbols: related_symbols_for_example(
                        example.example_id.as_str(),
                        &relation_lookup,
                        &analysis.symbols,
                    ),
                    related_modules: related_modules_for_example(
                        example.example_id.as_str(),
                        &relation_lookup,
                        &analysis.modules,
                    ),
                },
            )
        })
        .collect()
}

pub(super) fn raw_example_match_score(
    normalized_query: &str,
    example: &ExampleRecord,
    metadata: &ExampleSearchMetadata,
) -> Option<u8> {
    let title = example.title.to_ascii_lowercase();
    let path = example.path.to_ascii_lowercase();
    let summary = example
        .summary
        .as_deref()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    example_match_score(
        normalized_query,
        title.as_str(),
        path.as_str(),
        summary.as_str(),
        &metadata.related_symbols,
        &metadata.related_modules,
    )
}
