//! Modelica repository plugin entry point and capability registration.

use std::ffi::OsStr;
use std::path::Path;

use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, ModuleRecord, PluginAnalysisOutput, PluginLinkContext, PluginRegistry,
    RegisteredRepository, RelationRecord, RepoIntelligenceError, RepoIntelligencePlugin,
    RepoSourceFile, RepositoryAnalysisOutput, SymbolRecord,
};

use super::analysis;
use super::incremental::analyze_repo_owned_modelica_file_for_repository;
use super::parser_summary::fetch_modelica_parser_file_summary_blocking_for_repository;
use super::relations::build_incremental_doc_relations;

const MODELICA_PLUGIN_ID: &str = "modelica";

/// External Modelica analyzer for Repo Intelligence.
#[derive(Debug, Default, Clone, Copy)]
pub struct ModelicaRepoIntelligencePlugin;

/// Register the Modelica plugin into an existing Repo Intelligence registry.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the registry already contains a
/// plugin with the `modelica` identifier.
pub fn register_modelica_into(registry: &mut PluginRegistry) -> Result<(), RepoIntelligenceError> {
    registry.register(ModelicaRepoIntelligencePlugin)
}

inventory::submit! {
    xiuxian_wendao_core::repo_intelligence::BuiltinPluginRegistrar::new(
        MODELICA_PLUGIN_ID,
        register_modelica_into,
    )
}

impl RepoIntelligencePlugin for ModelicaRepoIntelligencePlugin {
    fn id(&self) -> &'static str {
        MODELICA_PLUGIN_ID
    }

    fn supports_repository(&self, repository: &RegisteredRepository) -> bool {
        repository
            .plugins
            .iter()
            .any(|plugin| plugin.id() == MODELICA_PLUGIN_ID)
    }

    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        if let Some(output) = analyze_repo_owned_modelica_file_for_repository(context, file)? {
            return Ok(output);
        }

        let summary = fetch_modelica_parser_file_summary_blocking_for_repository(
            &context.repository,
            file.path.as_str(),
            file.contents.as_str(),
        )?;
        let module_name = summary.class_name.unwrap_or_else(|| {
            std::path::Path::new(file.path.as_str())
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or("modelica")
                .to_string()
        });
        let module_id = format!("repo:{}:module:{module_name}", context.repository.id);
        let symbols = summary
            .declarations
            .into_iter()
            .map(|declaration| {
                let qualified_name = if declaration.name == module_name {
                    module_name.clone()
                } else {
                    format!("{module_name}.{}", declaration.name)
                };
                SymbolRecord {
                    repo_id: context.repository.id.clone(),
                    symbol_id: format!("repo:{}:symbol:{qualified_name}", context.repository.id),
                    module_id: Some(module_id.clone()),
                    name: declaration.name,
                    qualified_name,
                    kind: declaration.kind,
                    path: file.path.clone(),
                    line_start: declaration.line_start,
                    line_end: declaration.line_end,
                    signature: Some(declaration.signature),
                    audit_status: None,
                    verification_state: None,
                    attributes: declaration.attributes,
                }
            })
            .collect::<Vec<_>>();
        Ok(PluginAnalysisOutput {
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone(),
                module_id,
                qualified_name: module_name,
                path: file.path.clone(),
            }],
            symbols,
            imports: Vec::new(),
            examples: Vec::new(),
            docs: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn preflight_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<(), RepoIntelligenceError> {
        analysis::preflight_repository(context, repository_root)
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        analysis::analyze_repository(context, repository_root)
    }

    fn enrich_relations(
        &self,
        context: &PluginLinkContext,
    ) -> Result<Vec<RelationRecord>, RepoIntelligenceError> {
        Ok(build_incremental_doc_relations(
            context.repository.id.as_str(),
            &context.modules,
            &context.symbols,
            &context.docs,
        ))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/modelica_entry.rs"]
mod tests;
