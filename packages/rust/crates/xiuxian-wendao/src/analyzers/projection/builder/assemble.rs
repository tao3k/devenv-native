//! `analyzers::projection::builder::assemble` owns Wendao projection builder assemble behavior.

use std::collections::BTreeMap;

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{
    ProjectionInputBundle, ProjectionPageKind, ProjectionPageSeed, RelationKind,
};

use super::anchors::attach_target;
use super::anchors::{SourceAssociations, TargetAnchors};
use super::helpers::sorted_strings;
use super::kinds::{doc_projection_kind, projection_kind_token};
use super::sources::{
    attach_doc_source, attach_example_source, source_associations_for_module,
    source_associations_for_targets, symbol_ids_by_module,
};

/// Build deterministic projection inputs from Stage-1 Repo Intelligence output.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_projection_inputs(analysis: &RepositoryAnalysisOutput) -> ProjectionInputBundle {
    let repo_id = projection_repo_id(analysis);
    let symbol_ids_by_module = symbol_ids_by_module(analysis);
    let doc_lookup = analysis
        .docs
        .iter()
        .map(|doc| (doc.doc_id.to_string(), doc))
        .collect::<BTreeMap<_, _>>();
    let example_lookup = analysis
        .examples
        .iter()
        .map(|example| (example.example_id.to_string(), example))
        .collect::<BTreeMap<_, _>>();

    let mut docs_by_target = BTreeMap::<String, SourceAssociations>::new();
    let mut targets_by_doc = BTreeMap::<String, TargetAnchors>::new();
    let mut examples_by_target = BTreeMap::<String, SourceAssociations>::new();
    let mut targets_by_example = BTreeMap::<String, TargetAnchors>::new();

    for relation in &analysis.relations {
        match relation.kind {
            RelationKind::Documents => {
                if let Some(doc) = doc_lookup.get(&relation.source_id) {
                    attach_doc_source(
                        docs_by_target
                            .entry(relation.target_id.clone())
                            .or_default(),
                        doc,
                    );
                    attach_target(
                        targets_by_doc
                            .entry(relation.source_id.clone())
                            .or_default(),
                        &relation.target_id,
                    );
                }
            }
            RelationKind::ExampleOf => {
                if let Some(example) = example_lookup.get(&relation.source_id) {
                    attach_example_source(
                        examples_by_target
                            .entry(relation.target_id.clone())
                            .or_default(),
                        example,
                    );
                    attach_target(
                        targets_by_example
                            .entry(relation.source_id.clone())
                            .or_default(),
                        &relation.target_id,
                    );
                }
            }
            _ => {}
        }
    }

    let mut pages = Vec::new();

    for module in &analysis.modules {
        let docs = source_associations_for_module(
            &docs_by_target,
            module.module_id.as_str(),
            symbol_ids_by_module.get(module.module_id.as_str()),
        );
        let examples = source_associations_for_module(
            &examples_by_target,
            module.module_id.as_str(),
            symbol_ids_by_module.get(module.module_id.as_str()),
        );
        pages.push(ProjectionPageSeed {
            repo_id: repo_id.clone(),
            page_id: format!(
                "repo:{repo_id}:projection:reference:module:{}",
                module.module_id
            ),
            kind: ProjectionPageKind::Reference,
            title: module.qualified_name.clone(),
            module_ids: vec![module.module_id.to_string()],
            symbol_ids: Vec::new(),
            example_ids: examples.example_ids,
            doc_ids: docs.doc_ids,
            paths: sorted_strings(
                [module.path.to_string()],
                docs.doc_paths,
                examples.example_paths,
            ),
            format_hints: sorted_strings(Vec::<String>::new(), docs.format_hints, Vec::new()),
        });
    }

    for symbol in &analysis.symbols {
        let docs = docs_by_target
            .get(symbol.symbol_id.as_str())
            .cloned()
            .unwrap_or_default();
        let examples = examples_by_target
            .get(symbol.symbol_id.as_str())
            .cloned()
            .unwrap_or_default();
        pages.push(ProjectionPageSeed {
            repo_id: repo_id.clone(),
            page_id: format!(
                "repo:{repo_id}:projection:reference:symbol:{}",
                symbol.symbol_id
            ),
            kind: ProjectionPageKind::Reference,
            title: symbol.qualified_name.clone(),
            module_ids: symbol
                .module_id
                .as_ref()
                .map(ToString::to_string)
                .into_iter()
                .collect(),
            symbol_ids: vec![symbol.symbol_id.to_string()],
            example_ids: examples.example_ids,
            doc_ids: docs.doc_ids,
            paths: sorted_strings(
                [symbol.path.to_string()],
                docs.doc_paths,
                examples.example_paths,
            ),
            format_hints: sorted_strings(Vec::<String>::new(), docs.format_hints, Vec::new()),
        });
    }

    for example in &analysis.examples {
        let targets = targets_by_example
            .get(example.example_id.as_str())
            .cloned()
            .unwrap_or_default();
        let related_docs = source_associations_for_targets(&docs_by_target, &targets);
        pages.push(ProjectionPageSeed {
            repo_id: repo_id.clone(),
            page_id: format!(
                "repo:{repo_id}:projection:howto:example:{}",
                example.example_id
            ),
            kind: ProjectionPageKind::HowTo,
            title: example.title.clone(),
            module_ids: targets.module_ids,
            symbol_ids: targets.symbol_ids,
            example_ids: vec![example.example_id.to_string()],
            doc_ids: related_docs.doc_ids,
            paths: sorted_strings(
                [example.path.to_string()],
                related_docs.doc_paths,
                Vec::new(),
            ),
            format_hints: sorted_strings(
                Vec::<String>::new(),
                related_docs.format_hints,
                Vec::new(),
            ),
        });
    }

    for doc in &analysis.docs {
        let targets = targets_by_doc
            .get(doc.doc_id.as_str())
            .cloned()
            .unwrap_or_default();
        let related_examples = source_associations_for_targets(&examples_by_target, &targets);
        let kind = doc_projection_kind(doc, &targets);
        pages.push(ProjectionPageSeed {
            repo_id: repo_id.clone(),
            page_id: format!(
                "repo:{repo_id}:projection:{}:doc:{}",
                projection_kind_token(kind),
                doc.doc_id
            ),
            kind,
            title: doc.title.clone(),
            module_ids: targets.module_ids,
            symbol_ids: targets.symbol_ids,
            example_ids: related_examples.example_ids,
            doc_ids: vec![doc.doc_id.to_string()],
            paths: sorted_strings(
                [doc.path.to_string()],
                related_examples.example_paths,
                Vec::new(),
            ),
            format_hints: doc.format.clone().into_iter().collect(),
        });
    }

    pages.sort_by(|left, right| {
        projection_kind_token(left.kind)
            .cmp(projection_kind_token(right.kind))
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.page_id.cmp(&right.page_id))
    });

    ProjectionInputBundle { repo_id, pages }
}

fn projection_repo_id(analysis: &RepositoryAnalysisOutput) -> String {
    analysis
        .repository
        .as_ref()
        .map(|repository| repository.repo_id.to_string())
        .or_else(|| {
            analysis
                .modules
                .first()
                .map(|module| module.repo_id.to_string())
        })
        .or_else(|| {
            analysis
                .symbols
                .first()
                .map(|symbol| symbol.repo_id.to_string())
        })
        .or_else(|| {
            analysis
                .examples
                .first()
                .map(|example| example.repo_id.to_string())
        })
        .or_else(|| analysis.docs.first().map(|doc| doc.repo_id.to_string()))
        .unwrap_or_default()
}
