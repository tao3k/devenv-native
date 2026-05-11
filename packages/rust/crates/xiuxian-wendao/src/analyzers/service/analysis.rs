use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "search-runtime")]
use std::collections::BTreeSet;
#[cfg(feature = "search-runtime")]
use std::fs;

use xiuxian_git_repo::{MaterializedRepo, RepoSourceKind, SyncMode, discover_checkout_metadata};

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
    if !repository.has_repo_intelligence_plugins() {
        return Err(RepoIntelligenceError::MissingRepoIntelligencePlugins {
            repo_id: repository.id.clone().into(),
        });
    }

    let repository_source = resolve_target_file_analysis_source(repository, cwd)?;
    let repository_root = repository_source.checkout_root.clone();
    let checkout_metadata = discover_checkout_metadata(repository_root.as_path());
    if let Some(mut cached_output) =
        load_cached_target_file_analysis(repository, cwd, registry, repo_relative_path)?
    {
        finalize_target_file_analysis_output(
            repository,
            repository_root.as_path(),
            checkout_metadata.as_ref(),
            &mut cached_output,
        );
        return Ok(cached_output);
    }

    let analysis_context = AnalysisContext {
        repository: repository.clone(),
        repository_root: repository_root.clone(),
    };
    let plugins = registry.resolve_for_repository(repository)?;
    let source_path = repository_root.join(repo_relative_path);
    let source_text = fs::read_to_string(&source_path).map_err(|error| {
        RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "failed to read repository source `{}` for repo `{}`: {error}",
                source_path.display(),
                repository.id,
            ),
        }
    })?;
    let source_file = RepoSourceFile {
        path: repo_relative_path.to_string(),
        contents: source_text,
    };

    let mut output = RepositoryAnalysisOutput::default();
    let mut any_plugin_output = false;
    for plugin in &plugins {
        let plugin_output = plugin.analyze_file(&analysis_context, &source_file)?;
        any_plugin_output |= !(plugin_output.modules.is_empty()
            && plugin_output.symbols.is_empty()
            && plugin_output.examples.is_empty()
            && plugin_output.docs.is_empty()
            && plugin_output.diagnostics.is_empty());
        output.modules.extend(plugin_output.modules);
        output.symbols.extend(plugin_output.symbols);
        output.examples.extend(plugin_output.examples);
        output.docs.extend(plugin_output.docs);
        output.diagnostics.extend(plugin_output.diagnostics);
    }

    if !any_plugin_output {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` produced no file analysis output for `{repo_relative_path}`",
                repository.id,
            ),
        });
    }

    finalize_target_file_analysis_output(
        repository,
        repository_root.as_path(),
        checkout_metadata.as_ref(),
        &mut output,
    );
    for plugin in &plugins {
        let link_context = PluginLinkContext {
            repository: repository.clone(),
            repository_root: repository_root.clone(),
            modules: output.modules.clone(),
            symbols: output.symbols.clone(),
            examples: output.examples.clone(),
            docs: output.docs.clone(),
        };
        output
            .relations
            .extend(plugin.enrich_relations(&link_context)?);
    }
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
    if !repository.has_repo_intelligence_plugins() {
        return Err(RepoIntelligenceError::MissingRepoIntelligencePlugins {
            repo_id: repository.id.clone().into(),
        });
    }

    let repository_source = resolve_analysis_source(repository, cwd)?;
    let repository_root = repository_source.checkout_root.clone();
    let checkout_metadata = discover_checkout_metadata(repository_root.as_path());
    let cache_key = build_repository_analysis_cache_key(
        repository,
        &repository_source,
        checkout_metadata.as_ref(),
    );
    if let Some(cached) = load_cached_repository_analysis(&cache_key)? {
        return Ok(CachedRepositoryAnalysis {
            #[cfg(feature = "search-runtime")]
            cache_key,
            analysis: cached,
        });
    }

    let analysis_context = AnalysisContext {
        repository: repository.clone(),
        repository_root: repository_root.clone(),
    };
    let plugins = registry.resolve_for_repository(repository)?;
    preflight_repository_plugins(&plugins, &analysis_context, repository_root.as_path())?;

    let valkey_cache = ValkeyAnalysisCache::new()?;
    if let Some(cached) = load_cached_analysis_from_valkey(&cache_key, valkey_cache.as_ref())? {
        return Ok(CachedRepositoryAnalysis {
            #[cfg(feature = "search-runtime")]
            cache_key,
            analysis: cached,
        });
    }

    let mut output = analyze_repository_plugins(
        repository,
        repository_root.as_path(),
        &analysis_context,
        &plugins,
    )?;

    let link_context = PluginLinkContext {
        repository: repository.clone(),
        repository_root: repository_root.clone(),
        modules: output.modules.clone(),
        symbols: output.symbols.clone(),
        examples: output.examples.clone(),
        docs: output.docs.clone(),
    };
    enrich_repository_relations(&plugins, &link_context, &mut output)?;
    dedupe_relations(&mut output.relations);

    if output.repository.is_none() {
        output.repository = Some(repository.into());
    }
    if let Some(record) = output.repository.as_mut() {
        hydrate_repository_record(
            record,
            repository,
            repository_root.as_path(),
            checkout_metadata.as_ref(),
        );
    }

    let audit_results = skeptic::audit_symbols(&output.symbols, &output.docs, &output.relations);
    for symbol in &mut output.symbols {
        if let Some(state) = audit_results.get(symbol.symbol_id.as_str()) {
            symbol.verification_state = Some(state.clone().into());
        }
    }

    if let Some(ref cache) = valkey_cache {
        cache.set_analysis(RepositoryAnalysisValkeyScope::current(&cache_key), &output);
    }
    store_cached_repository_analysis(cache_key.clone(), &output)?;

    Ok(CachedRepositoryAnalysis {
        #[cfg(feature = "search-runtime")]
        cache_key,
        analysis: output,
    })
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
