//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::BTreeSet;

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{DocRecord, ModuleRecord, RelationKind, SymbolRecord};

pub(crate) fn resolve_module_scope<'a>(
    module_selector: Option<&str>,
    modules: &'a [ModuleRecord],
) -> Option<&'a ModuleRecord> {
    let selector = module_selector?.trim();
    if selector.is_empty() {
        return None;
    }

    modules.iter().find(|module| {
        module.module_id == selector || module.qualified_name == selector || module.path == selector
    })
}

pub(crate) fn docs_in_scope(
    scoped_module: Option<&ModuleRecord>,
    analysis: &RepositoryAnalysisOutput,
) -> Vec<DocRecord> {
    let mut docs = match scoped_module {
        None => analysis.docs.clone(),
        Some(module) => {
            let mut target_ids = BTreeSet::from([module.module_id.to_string()]);
            target_ids.extend(
                symbols_in_scope(Some(module), &analysis.symbols)
                    .into_iter()
                    .map(|symbol| symbol.symbol_id.to_string()),
            );
            let doc_ids = analysis
                .relations
                .iter()
                .filter(|relation| {
                    relation.kind == RelationKind::Documents
                        && target_ids.contains(relation.target_id.as_str())
                })
                .map(|relation| relation.source_id.clone())
                .collect::<BTreeSet<_>>();
            analysis
                .docs
                .iter()
                .filter(|doc| doc_ids.contains(doc.doc_id.as_str()))
                .cloned()
                .collect()
        }
    };
    let first_parser_doc = docs
        .iter()
        .position(|doc| doc.doc_target.is_some())
        .unwrap_or(docs.len());
    docs[..first_parser_doc].sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.doc_id.cmp(&right.doc_id))
    });
    docs
}

pub(crate) fn documented_symbol_ids(
    scoped_module: Option<&ModuleRecord>,
    symbols: &[SymbolRecord],
    relations: &[crate::analyzers::RelationRecord],
) -> BTreeSet<String> {
    let scoped_symbol_ids = symbols_in_scope(scoped_module, symbols)
        .into_iter()
        .map(|symbol| symbol.symbol_id.to_string())
        .collect::<BTreeSet<_>>();

    relations
        .iter()
        .filter(|relation| {
            relation.kind == RelationKind::Documents
                && scoped_symbol_ids.contains(relation.target_id.as_str())
        })
        .map(|relation| relation.target_id.clone())
        .collect()
}

pub(crate) fn symbols_in_scope<'a>(
    scoped_module: Option<&ModuleRecord>,
    symbols: &'a [SymbolRecord],
) -> Vec<&'a SymbolRecord> {
    match scoped_module {
        None => symbols.iter().collect(),
        Some(module) => symbols
            .iter()
            .filter(|symbol| symbol.module_id.as_deref() == Some(module.module_id.as_str()))
            .collect(),
    }
}
