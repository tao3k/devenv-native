//! `analyzers::service::search::coverage` owns Wendao service search coverage behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{DocCoverageQuery, DocCoverageResult};

use crate::analyzers::service::{
    analyze_repository_from_config_with_registry, bootstrap_builtin_registry,
};
use crate::analyzers::service::{
    docs_in_scope, documented_symbol_ids, repo_hierarchical_uri, resolve_module_scope,
    symbols_in_scope,
};

/// Build a documentation coverage result from normalized analysis records.
#[must_use]
pub fn build_doc_coverage(
    query: &DocCoverageQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocCoverageResult {
    let scoped_module = resolve_module_scope(query.module_id.as_deref(), &analysis.modules);
    let scoped_docs = docs_in_scope(scoped_module, analysis);
    let scoped_symbols = symbols_in_scope(scoped_module, &analysis.symbols);
    let covered_symbol_ids =
        documented_symbol_ids(scoped_module, &analysis.symbols, &analysis.relations);
    let covered_symbols = scoped_symbols
        .iter()
        .filter(|symbol| covered_symbol_ids.contains(symbol.symbol_id.as_str()))
        .count();

    DocCoverageResult {
        repo_id: query.repo_id.clone(),
        module_id: scoped_module
            .map(|module| module.module_id.to_string())
            .or_else(|| query.module_id.clone()),
        docs: scoped_docs,
        covered_symbols,
        uncovered_symbols: scoped_symbols.len().saturating_sub(covered_symbols),
        hierarchical_uri: Some(repo_hierarchical_uri(query.repo_id.as_str())),
        hierarchy: Some(vec!["repo".to_string(), query.repo_id.clone()]),
    }
}

/// Load configuration, analyze one repository, and return documentation coverage.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn doc_coverage_from_config_with_registry(
    query: &DocCoverageQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocCoverageResult, RepoIntelligenceError> {
    let analysis =
        analyze_repository_from_config_with_registry(&query.repo_id, config_path, cwd, registry)?;
    Ok(build_doc_coverage(query, &analysis))
}

/// Load configuration, analyze one repository, and return documentation coverage.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn doc_coverage_from_config(
    query: &DocCoverageQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocCoverageResult, RepoIntelligenceError> {
    let registry = bootstrap_builtin_registry()?;
    doc_coverage_from_config_with_registry(query, config_path, cwd, &registry)
}
