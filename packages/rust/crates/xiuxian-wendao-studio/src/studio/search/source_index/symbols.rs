use std::path::Path;

use walkdir::WalkDir;

use crate::studio::types::AstSearchHit;
use crate::studio::types::UiProjectConfig;
use xiuxian_wendao::dependency_indexer::extract_symbols;
use xiuxian_wendao::unified_symbol::{UnifiedSymbol, UnifiedSymbolIndex};

use super::filters::should_skip_entry;
use crate::studio::search::project_scope::{configured_project_scan_roots, index_path_for_entry};
use crate::studio::search::support::{infer_crate_name, source_language_label, symbol_kind_label};

pub(crate) fn build_symbol_index(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> UnifiedSymbolIndex {
    build_symbol_index_from_symbols(
        configured_project_scan_roots(config_root, projects)
            .into_iter()
            .flat_map(|root| {
                symbols_for_scan_root(project_root, root.as_path()).collect::<Vec<_>>()
            })
            .collect(),
    )
}

fn symbols_for_scan_root(project_root: &Path, root: &Path) -> impl Iterator<Item = UnifiedSymbol> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !should_skip_entry(entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| symbols_for_file(project_root, entry.path()))
        .flatten()
}

fn symbols_for_file(project_root: &Path, path: &Path) -> Option<Vec<UnifiedSymbol>> {
    let language = source_language_label(path)?;
    let normalized_path = index_path_for_entry(project_root, path);
    let crate_name = infer_crate_name(Path::new(normalized_path.as_str()));
    let symbols = extract_symbols(path, language).ok()?;
    Some(
        symbols
            .into_iter()
            .map(|symbol| {
                let location = format!("{normalized_path}:{}", symbol.line);
                UnifiedSymbol::new_project(
                    symbol.name.as_str(),
                    symbol_kind_label(&symbol.kind),
                    location.as_str(),
                    crate_name.as_str(),
                )
            })
            .collect(),
    )
}

pub(crate) fn build_symbol_index_from_ast_hits(hits: &[AstSearchHit]) -> UnifiedSymbolIndex {
    let symbols = hits
        .iter()
        .map(|hit| {
            let location = format!("{}:{}", hit.path, hit.line_start);
            UnifiedSymbol::new_project(
                hit.name.as_str(),
                hit.node_kind
                    .as_ref()
                    .map_or("symbol", |kind| kind.as_ref()),
                location.as_str(),
                hit.crate_name.as_str(),
            )
        })
        .collect();
    build_symbol_index_from_symbols(symbols)
}

fn build_symbol_index_from_symbols(symbols: Vec<UnifiedSymbol>) -> UnifiedSymbolIndex {
    let mut index = UnifiedSymbolIndex::new();
    index.add_symbols_batch(symbols);
    index
}
