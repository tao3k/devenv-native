use std::collections::BTreeMap;

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::contracts::{ProjectedPageRecord, ProjectionPageKind};
use crate::analyzers::{
    ProjectedGapKind, ProjectedGapRecord, ProjectedGapSummary, ProjectedGapSummaryEntry,
    RepoProjectedGapReportQuery, RepoProjectedGapReportResult,
};

use super::pages::build_projected_pages;

/// Build a deterministic deep-wiki projected gap report from repository truth.
#[must_use]
pub fn build_projected_gap_report(
    query: &RepoProjectedGapReportQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedGapReportResult {
    let pages = build_projected_pages(analysis);
    let lookup = ProjectedGapLookup::from_pages(pages.as_slice());
    let mut gaps = Vec::new();

    gaps.extend(module_gap_records(
        query.repo_id.as_str(),
        analysis,
        &lookup,
    ));
    gaps.extend(symbol_gap_records(
        query.repo_id.as_str(),
        analysis,
        &lookup,
    ));
    gaps.extend(example_gap_records(
        query.repo_id.as_str(),
        analysis,
        &lookup,
    ));
    gaps.extend(documentation_gap_records(
        query.repo_id.as_str(),
        analysis,
        &lookup,
    ));

    gaps.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.page_id.cmp(&right.page_id))
            .then_with(|| left.gap_id.cmp(&right.gap_id))
    });

    RepoProjectedGapReportResult {
        repo_id: query.repo_id.clone(),
        summary: build_gap_summary(gaps.as_slice(), pages.len()),
        gaps,
    }
}

struct ProjectedGapLookup {
    modules: BTreeMap<String, ProjectedPageRecord>,
    symbols: BTreeMap<String, ProjectedPageRecord>,
    examples: BTreeMap<String, ProjectedPageRecord>,
    docs: BTreeMap<String, ProjectedPageRecord>,
}

impl ProjectedGapLookup {
    fn from_pages(pages: &[ProjectedPageRecord]) -> Self {
        Self {
            modules: module_page_lookup(pages),
            symbols: symbol_page_lookup(pages),
            examples: example_page_lookup(pages),
            docs: doc_page_lookup(pages),
        }
    }
}

struct GapRecordInput<'a> {
    repo_id: &'a str,
    kind: ProjectedGapKind,
    page_kind: ProjectionPageKind,
    page: &'a ProjectedPageRecord,
    entity_id: &'a str,
}

fn module_gap_records(
    repo_id: &str,
    analysis: &RepositoryAnalysisOutput,
    lookup: &ProjectedGapLookup,
) -> Vec<ProjectedGapRecord> {
    analysis
        .modules
        .iter()
        .filter_map(|module| {
            let page = lookup.modules.get(module.module_id.as_str())?;
            page.doc_ids.is_empty().then(|| {
                build_gap_record(&GapRecordInput {
                    repo_id,
                    kind: ProjectedGapKind::ModuleReferenceWithoutDocumentation,
                    page_kind: ProjectionPageKind::Reference,
                    page,
                    entity_id: module.module_id.as_str(),
                })
            })
        })
        .collect()
}

fn symbol_gap_records(
    repo_id: &str,
    analysis: &RepositoryAnalysisOutput,
    lookup: &ProjectedGapLookup,
) -> Vec<ProjectedGapRecord> {
    let mut gaps = Vec::new();

    for symbol in &analysis.symbols {
        let Some(page) = lookup.symbols.get(symbol.symbol_id.as_str()) else {
            continue;
        };
        if page.doc_ids.is_empty() {
            gaps.push(build_gap_record(&GapRecordInput {
                repo_id,
                kind: ProjectedGapKind::SymbolReferenceWithoutDocumentation,
                page_kind: ProjectionPageKind::Reference,
                page,
                entity_id: symbol.symbol_id.as_str(),
            }));
            continue;
        }
        if symbol.verification_state.as_deref() == Some("unverified") {
            gaps.push(build_gap_record(&GapRecordInput {
                repo_id,
                kind: ProjectedGapKind::SymbolReferenceUnverified,
                page_kind: ProjectionPageKind::Reference,
                page,
                entity_id: symbol.symbol_id.as_str(),
            }));
        }
    }

    gaps
}

fn example_gap_records(
    repo_id: &str,
    analysis: &RepositoryAnalysisOutput,
    lookup: &ProjectedGapLookup,
) -> Vec<ProjectedGapRecord> {
    analysis
        .examples
        .iter()
        .filter_map(|example| {
            let page = lookup.examples.get(example.example_id.as_str())?;
            (page.module_ids.is_empty() && page.symbol_ids.is_empty()).then(|| {
                build_gap_record(&GapRecordInput {
                    repo_id,
                    kind: ProjectedGapKind::ExampleHowToWithoutAnchor,
                    page_kind: ProjectionPageKind::HowTo,
                    page,
                    entity_id: example.example_id.as_str(),
                })
            })
        })
        .collect()
}

fn documentation_gap_records(
    repo_id: &str,
    analysis: &RepositoryAnalysisOutput,
    lookup: &ProjectedGapLookup,
) -> Vec<ProjectedGapRecord> {
    analysis
        .docs
        .iter()
        .filter_map(|doc| {
            let page = lookup.docs.get(doc.doc_id.as_str())?;
            (page.module_ids.is_empty() && page.symbol_ids.is_empty()).then(|| {
                build_gap_record(&GapRecordInput {
                    repo_id,
                    kind: ProjectedGapKind::DocumentationPageWithoutAnchor,
                    page_kind: page.kind,
                    page,
                    entity_id: doc.doc_id.as_str(),
                })
            })
        })
        .collect()
}

fn build_gap_record(input: &GapRecordInput<'_>) -> ProjectedGapRecord {
    ProjectedGapRecord {
        repo_id: input.repo_id.to_owned(),
        gap_id: gap_id(input.repo_id, input.kind, input.entity_id),
        kind: input.kind,
        page_kind: input.page_kind,
        page_id: input.page.page_id.clone(),
        entity_id: input.entity_id.to_owned(),
        title: input.page.title.clone(),
        path: input.page.path.clone(),
        module_ids: input.page.module_ids.clone(),
        symbol_ids: input.page.symbol_ids.clone(),
        example_ids: input.page.example_ids.clone(),
        doc_ids: input.page.doc_ids.clone(),
        format_hints: input.page.format_hints.clone(),
    }
}

fn build_gap_summary(gaps: &[ProjectedGapRecord], page_count: usize) -> ProjectedGapSummary {
    let mut counts = BTreeMap::<ProjectedGapKind, usize>::new();
    for gap in gaps {
        *counts.entry(gap.kind).or_default() += 1;
    }

    ProjectedGapSummary {
        page_count,
        gap_count: gaps.len(),
        by_kind: counts
            .into_iter()
            .map(|(kind, count)| ProjectedGapSummaryEntry { kind, count })
            .collect(),
    }
}

fn module_page_lookup(pages: &[ProjectedPageRecord]) -> BTreeMap<String, ProjectedPageRecord> {
    pages
        .iter()
        .filter(|page| page.kind == ProjectionPageKind::Reference)
        .filter(|page| page.page_id.contains(":module:"))
        .filter_map(|page| {
            page.module_ids
                .first()
                .cloned()
                .map(|module_id| (module_id, page.clone()))
        })
        .collect()
}

fn symbol_page_lookup(pages: &[ProjectedPageRecord]) -> BTreeMap<String, ProjectedPageRecord> {
    pages
        .iter()
        .filter(|page| page.kind == ProjectionPageKind::Reference)
        .filter(|page| page.page_id.contains(":symbol:"))
        .filter_map(|page| {
            page.symbol_ids
                .first()
                .cloned()
                .map(|symbol_id| (symbol_id, page.clone()))
        })
        .collect()
}

fn example_page_lookup(pages: &[ProjectedPageRecord]) -> BTreeMap<String, ProjectedPageRecord> {
    pages
        .iter()
        .filter(|page| page.kind == ProjectionPageKind::HowTo)
        .filter(|page| page.page_id.contains(":example:"))
        .filter_map(|page| {
            page.example_ids
                .first()
                .cloned()
                .map(|example_id| (example_id, page.clone()))
        })
        .collect()
}

fn doc_page_lookup(pages: &[ProjectedPageRecord]) -> BTreeMap<String, ProjectedPageRecord> {
    pages
        .iter()
        .filter(|page| page.page_id.contains(":doc:"))
        .filter_map(|page| {
            page.doc_ids
                .first()
                .cloned()
                .map(|doc_id| (doc_id, page.clone()))
        })
        .collect()
}

fn gap_id(repo_id: &str, kind: ProjectedGapKind, entity_id: &str) -> String {
    format!(
        "repo:{repo_id}:projection-gap:{}:{entity_id}",
        gap_kind_token(kind)
    )
}

fn gap_kind_token(kind: ProjectedGapKind) -> &'static str {
    match kind {
        ProjectedGapKind::ModuleReferenceWithoutDocumentation => {
            "module_reference_without_documentation"
        }
        ProjectedGapKind::SymbolReferenceWithoutDocumentation => {
            "symbol_reference_without_documentation"
        }
        ProjectedGapKind::SymbolReferenceUnverified => "symbol_reference_unverified",
        ProjectedGapKind::ExampleHowToWithoutAnchor => "example_howto_without_anchor",
        ProjectedGapKind::DocumentationPageWithoutAnchor => "documentation_page_without_anchor",
    }
}
