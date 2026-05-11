use std::collections::BTreeSet;
use std::sync::Arc;

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::{DocRecord, RelationKind, RelationRecord};
use crate::analyzers::{PluginLinkContext, RepoIntelligencePlugin};

pub(super) fn rebuild_incremental_relations(
    repo_id: &str,
    link_context: &PluginLinkContext,
    existing_relations: &[RelationRecord],
    plugins: &[Arc<dyn RepoIntelligencePlugin>],
) -> Result<Vec<RelationRecord>, RepoIntelligenceError> {
    let mut relations = existing_relations
        .iter()
        .filter(|relation| is_preserved_incremental_relation(relation))
        .cloned()
        .collect::<Vec<_>>();
    relations.extend(build_generic_structural_relations(repo_id, link_context));
    relations.extend(build_docstring_relations(
        repo_id,
        &link_context.docs,
        link_context,
    ));
    for plugin in plugins {
        relations.extend(plugin.enrich_relations(link_context)?);
    }
    Ok(relations)
}

pub(crate) fn is_preserved_incremental_relation(relation: &RelationRecord) -> bool {
    relation.kind == RelationKind::Uses && relation.target_id.starts_with("external:")
}

fn build_generic_structural_relations(
    repo_id: &str,
    link_context: &PluginLinkContext,
) -> Vec<RelationRecord> {
    let repository_node_id = format!("repo:{repo_id}");
    let mut relations = Vec::new();

    relations.extend(link_context.modules.iter().map(|module| RelationRecord {
        repo_id: repo_id.to_string().into(),
        source_id: repository_node_id.clone(),
        target_id: module.module_id.to_string(),
        kind: RelationKind::Contains,
    }));
    relations.extend(link_context.symbols.iter().filter_map(|symbol| {
        symbol.module_id.as_ref().map(|module_id| RelationRecord {
            repo_id: repo_id.to_string().into(),
            source_id: module_id.to_string(),
            target_id: symbol.symbol_id.to_string(),
            kind: RelationKind::Declares,
        })
    }));
    relations.extend(link_context.examples.iter().map(|example| RelationRecord {
        repo_id: repo_id.to_string().into(),
        source_id: repository_node_id.clone(),
        target_id: example.example_id.to_string(),
        kind: RelationKind::Contains,
    }));
    relations.extend(link_context.docs.iter().map(|doc| RelationRecord {
        repo_id: repo_id.to_string().into(),
        source_id: repository_node_id.clone(),
        target_id: doc.doc_id.to_string(),
        kind: RelationKind::Contains,
    }));

    relations
}

fn build_docstring_relations(
    repo_id: &str,
    docs: &[DocRecord],
    link_context: &PluginLinkContext,
) -> Vec<RelationRecord> {
    docs.iter()
        .filter(|doc| doc.format.as_deref() == Some("julia_docstring"))
        .flat_map(|doc| {
            let mut target_ids: BTreeSet<String> = BTreeSet::new();
            if let Some(anchor) = doc.path.split('#').nth(1) {
                if let Some(module_name) = anchor.strip_prefix("module:") {
                    target_ids.extend(
                        link_context
                            .modules
                            .iter()
                            .filter(|module| {
                                module.qualified_name == module_name || module.path == doc.path
                            })
                            .map(|module| module.module_id.to_string()),
                    );
                }
                if let Some(symbol_name) = anchor.strip_prefix("symbol:") {
                    target_ids.extend(
                        link_context
                            .symbols
                            .iter()
                            .filter(|symbol| symbol.name == symbol_name)
                            .map(|symbol| symbol.symbol_id.to_string()),
                    );
                }
            }
            if target_ids.is_empty() {
                target_ids.extend(
                    link_context
                        .symbols
                        .iter()
                        .filter(|symbol| symbol.name == doc.title)
                        .map(|symbol| symbol.symbol_id.to_string()),
                );
            }
            target_ids
                .into_iter()
                .map(|target_id| RelationRecord {
                    repo_id: repo_id.to_string().into(),
                    source_id: doc.doc_id.to_string(),
                    target_id,
                    kind: RelationKind::Documents,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
