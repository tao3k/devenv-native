use std::path::Path;

use walkdir::WalkDir;

use crate::studio::types::SourceSymbolHit;
use crate::studio::types::UiProjectConfig;
use xiuxian_wendao::dependency_indexer::extract_dependency_symbols;
use xiuxian_wendao::unified_symbol::{UnifiedSymbol, UnifiedSymbolIndex};

use super::filters::should_skip_entry;
use super::navigation::source_symbol_navigation_target;
use crate::studio::search::project_scope::{
    configured_project_scan_roots, index_path_for_entry, project_metadata_for_path,
};
use crate::studio::search::support::{infer_crate_name, source_language_label, symbol_kind_label};

pub(crate) fn build_symbol_index(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> UnifiedSymbolIndex {
    let hits = build_source_symbol_hits(project_root, config_root, projects);
    build_symbol_index_from_source_symbol_hits(hits.as_slice())
}

pub(crate) fn build_source_symbol_hits(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> Vec<SourceSymbolHit> {
    configured_project_scan_roots(config_root, projects)
        .into_iter()
        .flat_map(|root| {
            source_symbol_hits_for_scan_root(project_root, config_root, projects, root.as_path())
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(crate) fn build_symbol_index_from_source_symbol_hits(
    hits: &[SourceSymbolHit],
) -> UnifiedSymbolIndex {
    build_symbol_index_from_symbols(
        hits.iter()
            .map(|hit| {
                let line = hit.line_start.max(1);
                let location = format!("{}:{line}", hit.path);
                UnifiedSymbol::new_project(
                    hit.name.as_str(),
                    source_symbol_kind(hit),
                    location.as_str(),
                    hit.crate_name.as_str(),
                )
            })
            .collect(),
    )
}

fn source_symbol_kind(hit: &SourceSymbolHit) -> &str {
    hit.node_kind.as_deref().unwrap_or("symbol")
}

fn source_symbol_hits_for_scan_root<'a>(
    project_root: &'a Path,
    config_root: &'a Path,
    projects: &'a [UiProjectConfig],
    root: &'a Path,
) -> impl Iterator<Item = SourceSymbolHit> + 'a {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !should_skip_entry(entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(move |entry| {
            source_symbol_hits_for_file(project_root, config_root, projects, entry.path())
        })
        .flatten()
}

fn source_symbol_hits_for_file(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
    path: &Path,
) -> Option<Vec<SourceSymbolHit>> {
    let language = source_language_label(path)?.to_string();
    let normalized_path = index_path_for_entry(project_root, path);
    let crate_name = infer_crate_name(Path::new(normalized_path.as_str()));
    let metadata = project_metadata_for_path(
        project_root,
        config_root,
        projects,
        normalized_path.as_str(),
    );
    let symbols = extract_dependency_symbols(path, language.as_str()).ok()?;
    Some(
        symbols
            .into_iter()
            .map(|symbol| {
                let line = symbol.line.max(1);
                let kind = symbol_kind_label(&symbol.kind);
                SourceSymbolHit {
                    name: symbol.name,
                    signature: format!("{kind} {}", normalized_path),
                    path: normalized_path.clone(),
                    language: language.clone(),
                    crate_name: crate_name.clone(),
                    project_name: metadata.project_name.clone(),
                    root_label: metadata.root_label.clone(),
                    node_kind: Some(kind.into()),
                    owner_title: None,
                    navigation_target: source_symbol_navigation_target(
                        normalized_path.as_str(),
                        crate_name.as_str(),
                        metadata.project_name.as_deref(),
                        metadata.root_label.as_deref(),
                        line,
                        line,
                    ),
                    line_start: line,
                    line_end: line,
                    score: 0.0,
                }
            })
            .collect(),
    )
}

fn build_symbol_index_from_symbols(symbols: Vec<UnifiedSymbol>) -> UnifiedSymbolIndex {
    let mut index = UnifiedSymbolIndex::new();
    index.add_symbols_batch(symbols);
    index
}
