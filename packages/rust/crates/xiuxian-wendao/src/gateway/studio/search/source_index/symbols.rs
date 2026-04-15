use std::path::Path;

use walkdir::WalkDir;

use crate::dependency_indexer::extract_symbols;
use crate::gateway::studio::types::AstSearchHit;
use crate::gateway::studio::types::UiProjectConfig;
use crate::unified_symbol::{UnifiedSymbol, UnifiedSymbolIndex};

use super::filters::should_skip_entry;
use crate::gateway::studio::search::project_scope::{
    configured_project_scan_roots, index_path_for_entry,
};
use crate::gateway::studio::search::support::{
    infer_crate_name, source_language_label, symbol_kind_label,
};

pub(crate) fn build_symbol_index(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> UnifiedSymbolIndex {
    let mut symbols = Vec::new();

    for root in configured_project_scan_roots(config_root, projects) {
        for entry in WalkDir::new(root.as_path())
            .into_iter()
            .filter_entry(|entry| !should_skip_entry(entry))
        {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_file() {
                continue;
            }

            let Some(language) = source_language_label(entry.path()) else {
                continue;
            };
            let normalized_path = index_path_for_entry(project_root, entry.path());
            let crate_name = infer_crate_name(Path::new(normalized_path.as_str()));

            if let Ok(extracted_symbols) = extract_symbols(entry.path(), language) {
                for symbol in extracted_symbols {
                    let location = format!("{normalized_path}:{}", symbol.line);
                    symbols.push(UnifiedSymbol::new_project(
                        symbol.name.as_str(),
                        symbol_kind_label(&symbol.kind),
                        location.as_str(),
                        crate_name.as_str(),
                    ));
                }
            }
        }
    }

    build_symbol_index_from_symbols(symbols)
}

pub(crate) fn build_symbol_index_from_ast_hits(hits: &[AstSearchHit]) -> UnifiedSymbolIndex {
    let symbols = hits
        .iter()
        .map(|hit| {
            let location = format!("{}:{}", hit.path, hit.line_start);
            UnifiedSymbol::new_project(
                hit.name.as_str(),
                hit.node_kind.as_deref().unwrap_or("symbol"),
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
