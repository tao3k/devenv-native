//! Repository analysis service orchestration and cache-aware entrypoints.

use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "search-runtime")]
use std::collections::BTreeSet;
#[cfg(feature = "search-runtime")]
use std::fs;

use xiuxian_git_repo::{MaterializedRepo, RepoSourceKind, SyncMode, discover_checkout_metadata};

#[cfg(feature = "search-runtime")]
use crate::analyzers::PluginAnalysisOutput;
use crate::analyzers::PluginRegistry;
use crate::analyzers::RegisteredRepository;
use crate::analyzers::RepoIntelligenceError;
#[cfg(feature = "search-runtime")]
use crate::analyzers::RepoSourceFile;
use crate::analyzers::cache::{
    RepositoryAnalysisCacheKey, RepositoryAnalysisValkeyScope, ValkeyAnalysisCache,
    build_repository_analysis_cache_key, load_cached_repository_analysis,
    store_cached_repository_analysis,
};
use crate::analyzers::resolve_registered_repository_source;
use crate::analyzers::skeptic;
use crate::analyzers::{
    AnalysisContext, PluginLinkContext, RepoIntelligencePlugin, RepositoryAnalysisOutput,
};
#[cfg(feature = "search-runtime")]
use crate::analyzers::{RelationKind, RelationRecord};

use super::bootstrap::bootstrap_builtin_registry;
use super::cached::CachedRepositoryAnalysis;
#[cfg(feature = "search-runtime")]
use super::cached::analyze_registered_repository_cached_bundle_with_registry;
use super::merge::{hydrate_repository_record, merge_repository_analysis};
use super::registry::load_registered_repository;
use super::relation_dedupe::dedupe_relations;

/// Analyze one repository from configuration into normalized records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when config loading or repository
/// analysis fails.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn analyze_repository_from_config_with_registry(
    repo_id: &str,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let repository = load_registered_repository(repo_id, config_path, cwd)?;
    analyze_registered_repository_with_registry(&repository, cwd, registry)
}

/// Analyze one repository from configuration into normalized records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when config loading or repository
/// analysis fails.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn analyze_repository_from_config(
    repo_id: &str,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let registry = bootstrap_builtin_registry()?;
    analyze_repository_from_config_with_registry(repo_id, config_path, cwd, &registry)
}

/// Analyze one already-resolved registered repository.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn analyze_registered_repository_with_registry(
    repository: &RegisteredRepository,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    analyze_registered_repository_bundle_with_registry(repository, cwd, registry)
        .map(|cached| cached.analysis)
}

#[cfg(feature = "search-runtime")]
/// Analyze one repository-relative file through configured plugins without
/// traversing the entire repository.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository source cannot be
/// resolved, the target file cannot be read, or a plugin file-analysis step
/// fails.
pub fn analyze_registered_repository_target_file_with_registry(
    repository: &RegisteredRepository,
    cwd: &Path,
    registry: &PluginRegistry,
    repo_relative_path: &str,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    ensure_repo_intelligence_plugins(repository)?;
    let target_context = prepare_target_file_analysis_context(repository, cwd)?;

    if let Some(mut cached_output) =
        load_cached_target_file_analysis(repository, cwd, registry, repo_relative_path)?
    {
        finalize_target_file_analysis_output(
            repository,
            target_context.repository_root.as_path(),
            target_context.checkout_metadata.as_ref(),
            &mut cached_output,
        );
        return Ok(cached_output);
    }

    let plugins = registry.resolve_for_repository(repository)?;
    let source_file = read_target_repo_source_file(
        repository,
        target_context.repository_root.as_path(),
        repo_relative_path,
    )?;
    let mut output = analyze_target_file_plugins(
        repository,
        &target_context.analysis_context,
        &plugins,
        &source_file,
    )?;

    finalize_target_file_analysis_output(
        repository,
        target_context.repository_root.as_path(),
        target_context.checkout_metadata.as_ref(),
        &mut output,
    );
    enrich_target_file_relations(&plugins, &target_context, &mut output)?;
    dedupe_relations(&mut output.relations);

    Ok(output)
}

/// Analyze one already-resolved registered repository and preserve its stable cache identity.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn analyze_registered_repository_bundle_with_registry(
    repository: &RegisteredRepository,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<CachedRepositoryAnalysis, RepoIntelligenceError> {
    ensure_repo_intelligence_plugins(repository)?;
    let repository_source = resolve_analysis_source(repository, cwd)?;
    let bundle_context = prepare_bundle_analysis_context(repository, &repository_source);
    let cache_key = build_repository_analysis_cache_key(
        repository,
        &repository_source,
        bundle_context.checkout_metadata.as_ref(),
    );
    if let Some(cached) = load_local_cached_bundle_analysis(&cache_key)? {
        return Ok(cached);
    }

    let plugins = resolve_preflight_repository_plugins(repository, registry, &bundle_context)?;
    let valkey_cache = ValkeyAnalysisCache::new()?;
    if let Some(cached) = load_cached_analysis_from_valkey(&cache_key, valkey_cache.as_ref())? {
        return Ok(cached_repository_analysis(cache_key, cached));
    }

    let output = analyze_uncached_repository_bundle(repository, &bundle_context, &plugins)?;
    store_repository_bundle_analysis(&cache_key, valkey_cache.as_ref(), &output)?;

    Ok(cached_repository_analysis(cache_key, output))
}

/// Analyze one already-resolved registered repository.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn analyze_registered_repository(
    repository: &RegisteredRepository,
    cwd: &Path,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let registry = bootstrap_builtin_registry()?;
    analyze_registered_repository_with_registry(repository, cwd, &registry)
}

fn resolve_analysis_source(
    repository: &RegisteredRepository,
    cwd: &Path,
) -> Result<MaterializedRepo, RepoIntelligenceError> {
    let status_source = resolve_registered_repository_source(repository, cwd, SyncMode::Status)?;
    if matches!(status_source.source_kind, RepoSourceKind::ManagedRemote)
        || !status_source.checkout_root.is_dir()
    {
        resolve_registered_repository_source(repository, cwd, SyncMode::Ensure)
    } else {
        Ok(status_source)
    }
}

fn ensure_repo_intelligence_plugins(
    repository: &RegisteredRepository,
) -> Result<(), RepoIntelligenceError> {
    if repository.has_repo_intelligence_plugins() {
        Ok(())
    } else {
        Err(RepoIntelligenceError::MissingRepoIntelligencePlugins {
            repo_id: repository.id.clone().into(),
        })
    }
}

struct BundleAnalysisContext {
    repository_root: std::path::PathBuf,
    checkout_metadata: Option<xiuxian_git_repo::LocalCheckoutMetadata>,
    analysis_context: AnalysisContext,
}

fn prepare_bundle_analysis_context(
    repository: &RegisteredRepository,
    repository_source: &MaterializedRepo,
) -> BundleAnalysisContext {
    let repository_root = repository_source.checkout_root.clone();
    let checkout_metadata = discover_checkout_metadata(repository_root.as_path());
    let analysis_context = AnalysisContext {
        repository: repository.clone(),
        repository_root: repository_root.clone(),
    };
    BundleAnalysisContext {
        repository_root,
        checkout_metadata,
        analysis_context,
    }
}

fn cached_repository_analysis(
    #[cfg(feature = "search-runtime")] cache_key: RepositoryAnalysisCacheKey,
    #[cfg(not(feature = "search-runtime"))] _cache_key: RepositoryAnalysisCacheKey,
    analysis: RepositoryAnalysisOutput,
) -> CachedRepositoryAnalysis {
    CachedRepositoryAnalysis {
        #[cfg(feature = "search-runtime")]
        cache_key,
        analysis,
    }
}

fn load_local_cached_bundle_analysis(
    cache_key: &RepositoryAnalysisCacheKey,
) -> Result<Option<CachedRepositoryAnalysis>, RepoIntelligenceError> {
    Ok(load_cached_repository_analysis(cache_key)?
        .map(|cached| cached_repository_analysis(cache_key.clone(), cached)))
}

fn resolve_preflight_repository_plugins(
    repository: &RegisteredRepository,
    registry: &PluginRegistry,
    bundle_context: &BundleAnalysisContext,
) -> Result<Vec<Arc<dyn RepoIntelligencePlugin>>, RepoIntelligenceError> {
    let plugins = registry.resolve_for_repository(repository)?;
    preflight_repository_plugins(
        &plugins,
        &bundle_context.analysis_context,
        bundle_context.repository_root.as_path(),
    )?;
    Ok(plugins)
}

fn analyze_uncached_repository_bundle(
    repository: &RegisteredRepository,
    bundle_context: &BundleAnalysisContext,
    plugins: &[Arc<dyn RepoIntelligencePlugin>],
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let mut output = analyze_repository_plugins(
        repository,
        bundle_context.repository_root.as_path(),
        &bundle_context.analysis_context,
        plugins,
    )?;

    let link_context = plugin_link_context(
        repository,
        bundle_context.repository_root.as_path(),
        &output,
    );
    enrich_repository_relations(plugins, &link_context, &mut output)?;
    dedupe_relations(&mut output.relations);
    finalize_bundle_analysis_output(repository, bundle_context, &mut output);
    Ok(output)
}

fn store_repository_bundle_analysis(
    cache_key: &RepositoryAnalysisCacheKey,
    valkey_cache: Option<&ValkeyAnalysisCache>,
    output: &RepositoryAnalysisOutput,
) -> Result<(), RepoIntelligenceError> {
    if let Some(cache) = valkey_cache {
        cache.set_analysis(RepositoryAnalysisValkeyScope::current(cache_key), output);
    }
    store_cached_repository_analysis(cache_key.clone(), output)
}

#[cfg(feature = "search-runtime")]
fn resolve_target_file_analysis_source(
    repository: &RegisteredRepository,
    cwd: &Path,
) -> Result<MaterializedRepo, RepoIntelligenceError> {
    let status_source = resolve_registered_repository_source(repository, cwd, SyncMode::Status)?;
    if status_source.checkout_root.is_dir() {
        Ok(status_source)
    } else {
        resolve_registered_repository_source(repository, cwd, SyncMode::Ensure)
    }
}

#[cfg(feature = "search-runtime")]
struct TargetFileAnalysisContext {
    repository_root: std::path::PathBuf,
    checkout_metadata: Option<xiuxian_git_repo::LocalCheckoutMetadata>,
    analysis_context: AnalysisContext,
}

#[cfg(feature = "search-runtime")]
fn prepare_target_file_analysis_context(
    repository: &RegisteredRepository,
    cwd: &Path,
) -> Result<TargetFileAnalysisContext, RepoIntelligenceError> {
    let repository_source = resolve_target_file_analysis_source(repository, cwd)?;
    let repository_root = repository_source.checkout_root.clone();
    let checkout_metadata = discover_checkout_metadata(repository_root.as_path());
    let analysis_context = AnalysisContext {
        repository: repository.clone(),
        repository_root: repository_root.clone(),
    };
    Ok(TargetFileAnalysisContext {
        repository_root,
        checkout_metadata,
        analysis_context,
    })
}

#[cfg(feature = "search-runtime")]
fn read_target_repo_source_file(
    repository: &RegisteredRepository,
    repository_root: &Path,
    repo_relative_path: &str,
) -> Result<RepoSourceFile, RepoIntelligenceError> {
    let source_path = repository_root.join(repo_relative_path);
    let contents = fs::read_to_string(&source_path).map_err(|error| {
        RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "failed to read repository source `{}` for repo `{}`: {error}",
                source_path.display(),
                repository.id,
            ),
        }
    })?;
    Ok(RepoSourceFile {
        path: repo_relative_path.to_string(),
        contents,
    })
}

#[cfg(feature = "search-runtime")]
fn analyze_target_file_plugins(
    repository: &RegisteredRepository,
    analysis_context: &AnalysisContext,
    plugins: &[Arc<dyn RepoIntelligencePlugin>],
    source_file: &RepoSourceFile,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let mut output = RepositoryAnalysisOutput::default();
    let mut any_plugin_output = false;
    for plugin in plugins {
        let plugin_output = plugin.analyze_file(analysis_context, source_file)?;
        any_plugin_output |= plugin_file_output_has_records(&plugin_output);
        output.modules.extend(plugin_output.modules);
        output.symbols.extend(plugin_output.symbols);
        output.examples.extend(plugin_output.examples);
        output.docs.extend(plugin_output.docs);
        output.diagnostics.extend(plugin_output.diagnostics);
    }

    if any_plugin_output {
        Ok(output)
    } else {
        Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` produced no file analysis output for `{}`",
                repository.id, source_file.path,
            ),
        })
    }
}

#[cfg(feature = "search-runtime")]
fn plugin_file_output_has_records(output: &PluginAnalysisOutput) -> bool {
    !(output.modules.is_empty()
        && output.symbols.is_empty()
        && output.examples.is_empty()
        && output.docs.is_empty()
        && output.diagnostics.is_empty())
}

#[cfg(feature = "search-runtime")]
fn enrich_target_file_relations(
    plugins: &[Arc<dyn RepoIntelligencePlugin>],
    target_context: &TargetFileAnalysisContext,
    output: &mut RepositoryAnalysisOutput,
) -> Result<(), RepoIntelligenceError> {
    let link_context = plugin_link_context(
        &target_context.analysis_context.repository,
        target_context.repository_root.as_path(),
        output,
    );
    for plugin in plugins {
        output
            .relations
            .extend(plugin.enrich_relations(&link_context)?);
    }
    Ok(())
}

#[cfg(feature = "search-runtime")]
fn load_cached_target_file_analysis(
    repository: &RegisteredRepository,
    cwd: &Path,
    registry: &PluginRegistry,
    repo_relative_path: &str,
) -> Result<Option<RepositoryAnalysisOutput>, RepoIntelligenceError> {
    match analyze_registered_repository_cached_bundle_with_registry(repository, cwd, registry) {
        Ok(cached) => {
            let filtered =
                filter_repository_analysis_to_target_path(cached.analysis, repo_relative_path);
            if target_file_analysis_has_records(&filtered) {
                Ok(Some(filtered))
            } else {
                Ok(None)
            }
        }
        Err(RepoIntelligenceError::PendingRepositoryIndex { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}

fn plugin_link_context(
    repository: &RegisteredRepository,
    repository_root: &Path,
    output: &RepositoryAnalysisOutput,
) -> PluginLinkContext {
    PluginLinkContext {
        repository: repository.clone(),
        repository_root: repository_root.to_path_buf(),
        modules: output.modules.clone(),
        symbols: output.symbols.clone(),
        examples: output.examples.clone(),
        docs: output.docs.clone(),
    }
}

fn finalize_bundle_analysis_output(
    repository: &RegisteredRepository,
    bundle_context: &BundleAnalysisContext,
    output: &mut RepositoryAnalysisOutput,
) {
    if output.repository.is_none() {
        output.repository = Some(repository.into());
    }
    if let Some(record) = output.repository.as_mut() {
        hydrate_repository_record(
            record,
            repository,
            bundle_context.repository_root.as_path(),
            bundle_context.checkout_metadata.as_ref(),
        );
    }

    let audit_results = skeptic::audit_symbols(&output.symbols, &output.docs, &output.relations);
    for symbol in &mut output.symbols {
        if let Some(state) = audit_results.get(symbol.symbol_id.as_str()) {
            symbol.verification_state = Some(state.clone().into());
        }
    }
}

#[cfg(feature = "search-runtime")]
fn filter_repository_analysis_to_target_path(
    analysis: RepositoryAnalysisOutput,
    repo_relative_path: &str,
) -> RepositoryAnalysisOutput {
    let modules = analysis
        .modules
        .into_iter()
        .filter(|module| module.path == repo_relative_path)
        .collect::<Vec<_>>();
    let module_ids = modules
        .iter()
        .map(|module| module.module_id.to_string())
        .collect::<BTreeSet<_>>();
    let symbols = analysis
        .symbols
        .into_iter()
        .filter(|symbol| {
            symbol.path == repo_relative_path
                || symbol
                    .module_id
                    .as_ref()
                    .is_some_and(|module_id| module_ids.contains(module_id.as_str()))
        })
        .collect::<Vec<_>>();
    let symbol_ids = symbols
        .iter()
        .map(|symbol| symbol.symbol_id.to_string())
        .collect::<BTreeSet<_>>();
    let imports = analysis
        .imports
        .into_iter()
        .filter(|import| module_ids.contains(import.module_id.as_str()))
        .collect::<Vec<_>>();
    let examples = analysis
        .examples
        .into_iter()
        .filter(|example| example.path == repo_relative_path)
        .collect::<Vec<_>>();
    let example_ids = examples
        .iter()
        .map(|example| example.example_id.to_string())
        .collect::<BTreeSet<_>>();
    let docs = analysis
        .docs
        .into_iter()
        .filter(|doc| doc.path == repo_relative_path)
        .collect::<Vec<_>>();
    let doc_ids = docs
        .iter()
        .map(|doc| doc.doc_id.to_string())
        .collect::<BTreeSet<_>>();
    let diagnostic_paths = [repo_relative_path, "package.mo"];
    let diagnostics = analysis
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic_paths.contains(&diagnostic.path.as_str()))
        .collect::<Vec<_>>();
    let kept_relation_ids = module_ids
        .iter()
        .chain(symbol_ids.iter())
        .chain(example_ids.iter())
        .chain(doc_ids.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let relations = analysis
        .relations
        .into_iter()
        .filter(|relation| {
            kept_relation_ids.contains(relation.source_id.as_str())
                && kept_relation_ids.contains(relation.target_id.as_str())
        })
        .collect::<Vec<_>>();

    RepositoryAnalysisOutput {
        repository: analysis.repository,
        modules,
        symbols,
        imports,
        examples,
        docs,
        relations,
        diagnostics,
    }
}

#[cfg(feature = "search-runtime")]
fn target_file_analysis_has_records(analysis: &RepositoryAnalysisOutput) -> bool {
    !(analysis.modules.is_empty()
        && analysis.symbols.is_empty()
        && analysis.imports.is_empty()
        && analysis.examples.is_empty()
        && analysis.docs.is_empty()
        && analysis.diagnostics.is_empty())
}

#[cfg(feature = "search-runtime")]
fn finalize_target_file_analysis_output(
    repository: &RegisteredRepository,
    repository_root: &Path,
    checkout_metadata: Option<&xiuxian_git_repo::LocalCheckoutMetadata>,
    output: &mut RepositoryAnalysisOutput,
) {
    let link_context = PluginLinkContext {
        repository: repository.clone(),
        repository_root: repository_root.to_path_buf(),
        modules: output.modules.clone(),
        symbols: output.symbols.clone(),
        examples: output.examples.clone(),
        docs: output.docs.clone(),
    };
    output
        .relations
        .extend(build_target_file_structural_relations(
            repository.id.as_str(),
            &link_context,
        ));
    dedupe_relations(&mut output.relations);

    if output.repository.is_none() {
        output.repository = Some(repository.into());
    }
    if let Some(record) = output.repository.as_mut() {
        hydrate_repository_record(record, repository, repository_root, checkout_metadata);
    }
}

fn preflight_repository_plugins(
    plugins: &[Arc<dyn RepoIntelligencePlugin>],
    analysis_context: &AnalysisContext,
    repository_root: &Path,
) -> Result<(), RepoIntelligenceError> {
    for plugin in plugins {
        plugin.preflight_repository(analysis_context, repository_root)?;
    }
    Ok(())
}

fn load_cached_analysis_from_valkey(
    cache_key: &RepositoryAnalysisCacheKey,
    valkey_cache: Option<&ValkeyAnalysisCache>,
) -> Result<Option<RepositoryAnalysisOutput>, RepoIntelligenceError> {
    let Some(cache) = valkey_cache else {
        return Ok(None);
    };
    let Some(cached) = cache.get_analysis(RepositoryAnalysisValkeyScope::current(cache_key)) else {
        return Ok(None);
    };
    store_cached_repository_analysis(cache_key.clone(), &cached)?;
    Ok(Some(cached))
}

fn analyze_repository_plugins(
    repository: &RegisteredRepository,
    repository_root: &Path,
    analysis_context: &AnalysisContext,
    plugins: &[Arc<dyn RepoIntelligencePlugin>],
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let mut output = RepositoryAnalysisOutput::default();
    let mut any_plugin_output = false;

    for plugin in plugins {
        let plugin_output = plugin.analyze_repository(analysis_context, repository_root)?;
        any_plugin_output = true;
        merge_repository_analysis(&mut output, plugin_output);
    }

    if any_plugin_output {
        Ok(output)
    } else {
        Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` produced no repository analysis output",
                repository.id
            ),
        })
    }
}

fn enrich_repository_relations(
    plugins: &[Arc<dyn RepoIntelligencePlugin>],
    link_context: &PluginLinkContext,
    output: &mut RepositoryAnalysisOutput,
) -> Result<(), RepoIntelligenceError> {
    for plugin in plugins {
        output
            .relations
            .extend(plugin.enrich_relations(link_context)?);
    }
    Ok(())
}

#[cfg(feature = "search-runtime")]
fn build_target_file_structural_relations(
    repo_id: &str,
    link_context: &PluginLinkContext,
) -> Vec<RelationRecord> {
    let repository_node_id = format!("repo:{repo_id}");
    let mut relations = link_context
        .modules
        .iter()
        .map(|module| RelationRecord {
            repo_id: repo_id.to_string().into(),
            source_id: repository_node_id.clone(),
            target_id: module.module_id.to_string(),
            kind: RelationKind::Contains,
        })
        .collect::<Vec<_>>();
    relations.extend(link_context.symbols.iter().filter_map(|symbol| {
        symbol.module_id.as_ref().map(|module_id| RelationRecord {
            repo_id: repo_id.to_string().into(),
            source_id: module_id.to_string(),
            target_id: symbol.symbol_id.to_string(),
            kind: RelationKind::Contains,
        })
    }));
    relations
}
