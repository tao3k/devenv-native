use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use xiuxian_git_repo::LocalCheckoutMetadata;

use crate::analyzers::RegisteredRepository;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryRecord;
use crate::analyzers::{
    AnalysisContext, PluginAnalysisOutput, PluginLinkContext, RepoIntelligencePlugin,
    RepoSourceFile, RepositoryAnalysisOutput,
};

pub(crate) struct IncrementalApplyContext<'a> {
    pub(crate) repository: &'a RegisteredRepository,
    pub(crate) repository_root: &'a Path,
    pub(crate) checkout_metadata: Option<&'a LocalCheckoutMetadata>,
    pub(crate) plugins: &'a [Arc<dyn RepoIntelligencePlugin>],
}

pub(crate) fn apply_incremental_plugin_outputs(
    context: &IncrementalApplyContext<'_>,
    base: &mut RepositoryAnalysisOutput,
    overlays: Vec<PluginAnalysisOutput>,
    changed_paths: &BTreeSet<String>,
    deleted_paths: &BTreeSet<String>,
) -> Result<(), RepoIntelligenceError> {
    for overlay in overlays {
        super::merge::replace_records_for_paths(base, overlay, changed_paths, deleted_paths);
    }

    refresh_repository_record(
        base,
        context.repository,
        context.repository_root,
        context.checkout_metadata,
    );
    let link_context = PluginLinkContext {
        repository: context.repository.clone(),
        repository_root: context.repository_root.to_path_buf(),
        modules: base.modules.clone(),
        symbols: base.symbols.clone(),
        examples: base.examples.clone(),
        docs: base.docs.clone(),
    };
    let mut relations = super::relations::rebuild_incremental_relations(
        context.repository.id.as_str(),
        &link_context,
        &base.relations,
        context.plugins,
    )?;
    crate::analyzers::service::relation_dedupe::dedupe_relations(&mut relations);
    base.relations = relations;
    Ok(())
}

pub(crate) fn analyze_changed_files(
    analysis_context: &AnalysisContext,
    plugin: &Arc<dyn RepoIntelligencePlugin>,
    files: &[RepoSourceFile],
) -> Result<Vec<PluginAnalysisOutput>, RepoIntelligenceError> {
    files
        .iter()
        .map(|file| plugin.analyze_file(analysis_context, file))
        .collect()
}

fn refresh_repository_record(
    analysis: &mut RepositoryAnalysisOutput,
    repository: &RegisteredRepository,
    repository_root: &Path,
    checkout_metadata: Option<&LocalCheckoutMetadata>,
) {
    if analysis.repository.is_none() {
        analysis.repository = Some(RepositoryRecord::from(repository));
    }
    if let Some(record) = analysis.repository.as_mut() {
        crate::analyzers::service::merge::hydrate_repository_record(
            record,
            repository,
            repository_root,
            checkout_metadata,
        );
        record.revision = checkout_metadata.and_then(|metadata| metadata.revision.clone());
        if record.path.trim().is_empty() {
            record.path = repository_root.display().to_string();
        }
        if record.url.is_none() {
            record.url.clone_from(&repository.url);
        }
    }
}
