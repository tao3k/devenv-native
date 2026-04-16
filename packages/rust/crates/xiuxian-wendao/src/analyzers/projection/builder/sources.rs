use std::collections::BTreeMap;

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{DocRecord, ExampleRecord};

use super::anchors::{SourceAssociations, TargetAnchors};
use super::helpers::push_unique;

pub(super) fn symbol_ids_by_module(
    analysis: &RepositoryAnalysisOutput,
) -> BTreeMap<String, Vec<String>> {
    let mut symbol_ids = BTreeMap::<String, Vec<String>>::new();
    for symbol in &analysis.symbols {
        let Some(module_id) = symbol.module_id.as_ref() else {
            continue;
        };
        push_unique(
            symbol_ids.entry(module_id.clone()).or_default(),
            symbol.symbol_id.clone(),
        );
    }
    symbol_ids
}

pub(super) fn source_associations_for_module(
    by_target: &BTreeMap<String, SourceAssociations>,
    module_id: &str,
    symbol_ids: Option<&Vec<String>>,
) -> SourceAssociations {
    let mut target_ids = vec![module_id.to_string()];
    if let Some(symbol_ids) = symbol_ids {
        for symbol_id in symbol_ids {
            push_unique(&mut target_ids, symbol_id.clone());
        }
    }
    source_associations_for_target_ids(by_target, &target_ids)
}

pub(super) fn source_associations_for_targets(
    by_target: &BTreeMap<String, SourceAssociations>,
    targets: &TargetAnchors,
) -> SourceAssociations {
    let mut target_ids = Vec::new();
    for module_id in &targets.module_ids {
        push_unique(&mut target_ids, module_id.clone());
    }
    for symbol_id in &targets.symbol_ids {
        push_unique(&mut target_ids, symbol_id.clone());
    }
    source_associations_for_target_ids(by_target, &target_ids)
}

pub(super) fn source_associations_for_target_ids(
    by_target: &BTreeMap<String, SourceAssociations>,
    target_ids: &[String],
) -> SourceAssociations {
    let mut merged = SourceAssociations::default();
    for target_id in target_ids {
        let Some(associations) = by_target.get(target_id) else {
            continue;
        };
        for doc_id in &associations.doc_ids {
            push_unique(&mut merged.doc_ids, doc_id.clone());
        }
        for example_id in &associations.example_ids {
            push_unique(&mut merged.example_ids, example_id.clone());
        }
        for doc_path in &associations.doc_paths {
            push_unique(&mut merged.doc_paths, doc_path.clone());
        }
        for example_path in &associations.example_paths {
            push_unique(&mut merged.example_paths, example_path.clone());
        }
        for format_hint in &associations.format_hints {
            push_unique(&mut merged.format_hints, format_hint.clone());
        }
    }
    merged
}

pub(super) fn attach_doc_source(associations: &mut SourceAssociations, doc: &DocRecord) {
    push_unique(&mut associations.doc_ids, doc.doc_id.clone());
    push_unique(&mut associations.doc_paths, doc.path.clone());
    if let Some(format) = &doc.format {
        push_unique(&mut associations.format_hints, format.clone());
    }
}

pub(super) fn attach_example_source(
    associations: &mut SourceAssociations,
    example: &ExampleRecord,
) {
    push_unique(&mut associations.example_ids, example.example_id.clone());
    push_unique(&mut associations.example_paths, example.path.clone());
}
