//! Owns the Studio search handlers index surface.

use std::path::Path;

use crate::studio::search::source_index;
use crate::studio::types::AstSearchHit;
use crate::studio::types::UiProjectConfig;
use xiuxian_wendao::unified_symbol::UnifiedSymbolIndex;

/// Build Studio AST search hits from configured project roots.
#[must_use]
pub fn build_ast_index(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> Vec<AstSearchHit> {
    source_index::build_ast_index(project_root, config_root, projects)
}

/// Build Studio symbol search indices from configured project roots.
#[must_use]
pub fn build_symbol_index(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> UnifiedSymbolIndex {
    source_index::build_symbol_index(project_root, config_root, projects)
}
