//! `analyzers::projection::pages` owns Wendao analyzers projection pages behavior.

use std::collections::{BTreeMap, BTreeSet};

use crate::analyzers::RepositoryAnalysisOutput;

use super::builder::build_projection_inputs;
use super::contracts::{
    ProjectedPageRecord, ProjectedPageSection, ProjectionInputBundle, ProjectionPageKind,
    ProjectionPageSeed,
};

#[derive(Clone)]
struct DisplayItem {
    title: String,
    path: String,
}

#[derive(Default)]
struct ProjectionDisplayContext {
    modules: BTreeMap<String, DisplayItem>,
    symbols: BTreeMap<String, DisplayItem>,
    examples: BTreeMap<String, DisplayItem>,
    docs: BTreeMap<String, DisplayItem>,
}

/// Build deterministic projected page records from Repo Intelligence output.
#[must_use]
pub fn build_projected_pages(analysis: &RepositoryAnalysisOutput) -> Vec<ProjectedPageRecord> {
    let bundle = build_projection_inputs(analysis);
    let context = projection_display_context(analysis);
    build_projected_pages_from_bundle(&bundle, &context)
}

fn build_projected_pages_from_bundle(
    bundle: &ProjectionInputBundle,
    context: &ProjectionDisplayContext,
) -> Vec<ProjectedPageRecord> {
    bundle
        .pages
        .iter()
        .map(|seed| {
            let doc_id = seed.doc_ids.first().cloned().unwrap_or_default();
            let path = seed.paths.first().cloned().unwrap_or_default();
            let mut keywords = vec![seed.title.clone(), path.clone(), doc_id.clone()];
            keywords.extend(seed.format_hints.iter().cloned());
            keywords.sort();
            keywords.dedup();

            ProjectedPageRecord {
                repo_id: seed.repo_id.clone(),
                page_id: seed.page_id.clone(),
                kind: seed.kind,
                title: seed.title.clone(),
                module_ids: seed.module_ids.clone(),
                symbol_ids: seed.symbol_ids.clone(),
                example_ids: seed.example_ids.clone(),
                doc_ids: seed.doc_ids.clone(),
                paths: seed.paths.clone(),
                format_hints: seed.format_hints.clone(),
                sections: build_sections(seed, context),
                doc_id,
                path,
                keywords,
            }
        })
        .collect()
}

fn build_sections(
    seed: &ProjectionPageSeed,
    context: &ProjectionDisplayContext,
) -> Vec<ProjectedPageSection> {
    let anchor_lines = render_anchor_lines(seed, context);
    let doc_items = lookup_items(&seed.doc_ids, &context.docs);
    let example_items = lookup_items(&seed.example_ids, &context.examples);

    let mut sections = vec![ProjectedPageSection {
        section_id: format!("{}#overview", seed.page_id),
        title: "Overview".to_string(),
        level: 1,
        body: render_overview_body(seed),
        paths: seed.paths.clone(),
    }];

    if !anchor_lines.is_empty() {
        sections.push(ProjectedPageSection {
            section_id: format!("{}#anchors", seed.page_id),
            title: "Anchors".to_string(),
            level: 2,
            body: anchor_lines.join("\n"),
            paths: sorted_paths(
                lookup_paths(&seed.module_ids, &context.modules),
                lookup_paths(&seed.symbol_ids, &context.symbols),
            ),
        });
    }

    if !doc_items.is_empty() {
        sections.push(ProjectedPageSection {
            section_id: format!("{}#sources", seed.page_id),
            title: doc_section_title(seed.kind).to_string(),
            level: 2,
            body: render_item_lines("Documentation", &doc_items).join("\n"),
            paths: doc_items.into_iter().map(|item| item.path).collect(),
        });
    }

    if !example_items.is_empty() {
        sections.push(ProjectedPageSection {
            section_id: format!("{}#examples", seed.page_id),
            title: "Examples".to_string(),
            level: 2,
            body: render_item_lines("Examples", &example_items).join("\n"),
            paths: example_items.into_iter().map(|item| item.path).collect(),
        });
    }

    sections
}

fn render_overview_body(seed: &ProjectionPageSeed) -> String {
    let mut lines = vec![format!(
        "Projected {} page for `{}`.",
        projection_kind_label(seed.kind),
        seed.title
    )];
    if !seed.format_hints.is_empty() {
        lines.push("Format hints:".to_string());
        lines.extend(seed.format_hints.iter().map(|hint| format!("- `{hint}`")));
    }
    if !seed.paths.is_empty() {
        lines.push("Source paths:".to_string());
        lines.extend(seed.paths.iter().map(|path| format!("- `{path}`")));
    }
    lines.join("\n")
}

fn render_anchor_lines(
    seed: &ProjectionPageSeed,
    context: &ProjectionDisplayContext,
) -> Vec<String> {
    let module_items = lookup_items(&seed.module_ids, &context.modules);
    let symbol_items = lookup_items(&seed.symbol_ids, &context.symbols);
    let mut lines = Vec::new();
    if !module_items.is_empty() {
        lines.extend(render_item_lines("Modules", &module_items));
    }
    if !symbol_items.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(render_item_lines("Symbols", &symbol_items));
    }
    lines
}

fn render_item_lines(label: &str, items: &[DisplayItem]) -> Vec<String> {
    let mut lines = vec![format!("{label}:")];
    lines.extend(
        items
            .iter()
            .map(|item| format!("- `{}` ({})", item.title, item.path)),
    );
    lines
}

fn projection_display_context(analysis: &RepositoryAnalysisOutput) -> ProjectionDisplayContext {
    ProjectionDisplayContext {
        modules: analysis
            .modules
            .iter()
            .map(|module| {
                (
                    module.module_id.to_string(),
                    DisplayItem {
                        title: module.qualified_name.clone(),
                        path: module.path.to_string(),
                    },
                )
            })
            .collect(),
        symbols: analysis
            .symbols
            .iter()
            .map(|symbol| {
                (
                    symbol.symbol_id.to_string(),
                    DisplayItem {
                        title: symbol.qualified_name.clone(),
                        path: symbol.path.to_string(),
                    },
                )
            })
            .collect(),
        examples: analysis
            .examples
            .iter()
            .map(|example| {
                (
                    example.example_id.to_string(),
                    DisplayItem {
                        title: example.title.clone(),
                        path: example.path.to_string(),
                    },
                )
            })
            .collect(),
        docs: analysis
            .docs
            .iter()
            .map(|doc| {
                (
                    doc.doc_id.to_string(),
                    DisplayItem {
                        title: doc.title.clone(),
                        path: doc.path.to_string(),
                    },
                )
            })
            .collect(),
    }
}

fn lookup_items(ids: &[String], lookup: &BTreeMap<String, DisplayItem>) -> Vec<DisplayItem> {
    let mut items = ids
        .iter()
        .filter_map(|id| lookup.get(id).cloned())
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.title
            .cmp(&right.title)
            .then_with(|| left.path.cmp(&right.path))
    });
    items
}

fn lookup_paths(ids: &[String], lookup: &BTreeMap<String, DisplayItem>) -> Vec<String> {
    ids.iter()
        .filter_map(|id| lookup.get(id).map(|item| item.path.clone()))
        .collect()
}

fn sorted_paths<I, J>(left: I, right: J) -> Vec<String>
where
    I: IntoIterator<Item = String>,
    J: IntoIterator<Item = String>,
{
    let mut values = BTreeSet::new();
    values.extend(left);
    values.extend(right);
    values.into_iter().collect()
}

fn doc_section_title(kind: ProjectionPageKind) -> &'static str {
    match kind {
        ProjectionPageKind::Reference => "Documentation",
        ProjectionPageKind::HowTo => "Supporting Docs",
        ProjectionPageKind::Tutorial | ProjectionPageKind::Explanation => "Sources",
    }
}

fn projection_kind_label(kind: ProjectionPageKind) -> &'static str {
    match kind {
        ProjectionPageKind::Reference => "reference",
        ProjectionPageKind::HowTo => "how-to",
        ProjectionPageKind::Tutorial => "tutorial",
        ProjectionPageKind::Explanation => "explanation",
    }
}
