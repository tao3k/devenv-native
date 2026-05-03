pub(super) use std::collections::{BTreeMap, HashSet};
pub(super) use std::fs;
pub(super) use std::path::PathBuf;
pub(super) use std::sync::Arc;
pub(super) use std::time::Duration;

pub(super) use crate::analyzers::PluginRegistry;
pub(super) use crate::analyzers::RepoSourceFile;
pub(super) use crate::analyzers::{
    AnalysisContext, PluginAnalysisOutput, RegisteredRepository, RepoIntelligenceError,
    RepoIntelligencePlugin, RepositoryAnalysisOutput, RepositoryPluginConfig,
    RepositoryRefreshPolicy, analyze_registered_repository_with_registry,
    bootstrap_builtin_registry,
};
pub(super) use crate::analyzers::{ModuleRecord, RepoSymbolKind, RepositoryRecord, SymbolRecord};
pub(super) use crate::analyzers::{RepoSourceKind, RepoSyncResult};
pub(super) use crate::repo_index::state::coordinator::PreparedIncrementalAnalysis;
pub(super) use crate::repo_index::state::fingerprint::timestamp_now;
pub(super) use crate::repo_index::state::tests::{new_coordinator, new_coordinator_with_registry};
pub(super) use crate::repo_index::types::RepoIndexEntryStatus;
pub(super) use crate::search::{
    RepoSearchAvailability, SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneCache, SearchPlanePhase, SearchPlaneService, SearchPublicationStorageFormat,
    SearchRepoPublicationInput,
};
pub(super) use crate::test_support::linked_parser_summary::ensure_linked_modelica_parser_summary_service;
use crate::test_support::linked_parser_summary::linked_parser_summary_base_url;
pub(super) use crate::test_support::{commit_all, init_git_repository};
pub(super) use chrono::Utc;
pub(super) use xiuxian_git_repo::discover_checkout_metadata;
pub(super) struct LinkedParserSummaryTestGuard {
    killed: bool,
}

impl LinkedParserSummaryTestGuard {
    pub(super) fn kill(&mut self) {
        self.killed = true;
    }
}

pub(super) fn spawn_wendaosearch_julia_parser_summary_service()
-> (String, LinkedParserSummaryTestGuard) {
    (
        linked_parser_summary_base_url()
            .unwrap_or_else(|error| panic!("linked Julia parser-summary service: {error}")),
        LinkedParserSummaryTestGuard { killed: false },
    )
}

pub(super) fn spawn_wendaosearch_modelica_parser_summary_service()
-> (String, LinkedParserSummaryTestGuard) {
    (
        linked_parser_summary_base_url()
            .unwrap_or_else(|error| panic!("linked Modelica parser-summary service: {error}")),
        LinkedParserSummaryTestGuard { killed: false },
    )
}

pub(super) fn julia_parser_summary_plugin_config(base_url: &str) -> RepositoryPluginConfig {
    RepositoryPluginConfig::Config {
        id: "julia".to_string(),
        options: serde_json::json!({
            "parser_summary_transport": {
                "base_url": base_url,
                "file_summary": {
                    "schema_version": "v3"
                },
                "root_summary": {
                    "schema_version": "v3"
                }
            }
        }),
    }
}

pub(super) fn modelica_parser_summary_plugin_config(base_url: &str) -> RepositoryPluginConfig {
    RepositoryPluginConfig::Config {
        id: "modelica".to_string(),
        options: serde_json::json!({
            "parser_summary_transport": {
                "base_url": base_url,
                "file_summary": {
                    "schema_version": "v3"
                }
            }
        }),
    }
}

pub(super) fn mixed_julia_modelica_plugin_configs(
    julia_base_url: &str,
    modelica_base_url: &str,
) -> Vec<RepositoryPluginConfig> {
    vec![
        julia_parser_summary_plugin_config(julia_base_url),
        modelica_parser_summary_plugin_config(modelica_base_url),
    ]
}

pub(super) fn mixed_modelica_rust_plugin_configs() -> Vec<RepositoryPluginConfig> {
    vec![
        RepositoryPluginConfig::Id("modelica".to_string()),
        RepositoryPluginConfig::Id("rust".to_string()),
    ]
}

pub(super) fn mixed_rust_unknown_plugin_configs() -> Vec<RepositoryPluginConfig> {
    vec![
        RepositoryPluginConfig::Id("rust".to_string()),
        RepositoryPluginConfig::Id("ast-grep".to_string()),
    ]
}

pub(super) fn mixed_modelica_unknown_plugin_configs() -> Vec<RepositoryPluginConfig> {
    vec![
        RepositoryPluginConfig::Id("modelica".to_string()),
        RepositoryPluginConfig::Id("ast-grep".to_string()),
    ]
}

#[derive(Clone)]
pub(super) struct RuntimeRustPlugin;

impl RepoIntelligencePlugin for RuntimeRustPlugin {
    fn id(&self) -> &'static str {
        "rust"
    }

    fn supports_repository(&self, _repository: &RegisteredRepository) -> bool {
        true
    }

    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        let module_id = format!("repo:{}:module:fixture", context.repository.id);
        Ok(PluginAnalysisOutput {
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone(),
                module_id: module_id.clone(),
                qualified_name: "fixture".to_string(),
                path: file.path.clone(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone(),
                symbol_id: format!("repo:{}:symbol:solve", context.repository.id),
                module_id: Some(module_id),
                name: "solve".to_string(),
                qualified_name: "fixture.solve".to_string(),
                kind: RepoSymbolKind::Function,
                path: file.path.clone(),
                line_start: Some(1),
                line_end: Some(1),
                signature: Some("solve(x)".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            imports: Vec::new(),
            examples: Vec::new(),
            docs: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &std::path::Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        Ok(RepositoryAnalysisOutput {
            repository: Some(RepositoryRecord {
                repo_id: context.repository.id.clone(),
                name: "fixture".to_string(),
                path: repository_root.display().to_string(),
                url: None,
                revision: None,
                version: None,
                uuid: None,
                dependencies: Vec::new(),
            }),
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone(),
                module_id: format!("repo:{}:module:fixture", context.repository.id),
                qualified_name: "fixture".to_string(),
                path: "src/lib.rs".to_string(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone(),
                symbol_id: format!("repo:{}:symbol:solve", context.repository.id),
                module_id: Some(format!("repo:{}:module:fixture", context.repository.id)),
                name: "solve".to_string(),
                qualified_name: "fixture.solve".to_string(),
                kind: RepoSymbolKind::Function,
                path: "src/lib.rs".to_string(),
                line_start: Some(1),
                line_end: Some(1),
                signature: Some("solve(x)".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            ..RepositoryAnalysisOutput::default()
        })
    }
}

#[derive(Clone)]
pub(super) struct RuntimeModelicaPlugin;

impl RepoIntelligencePlugin for RuntimeModelicaPlugin {
    fn id(&self) -> &'static str {
        "modelica"
    }

    fn supports_repository(&self, _repository: &RegisteredRepository) -> bool {
        true
    }

    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        let module_id = format!("repo:{}:module:DemoLib", context.repository.id);
        Ok(PluginAnalysisOutput {
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone(),
                module_id: module_id.clone(),
                qualified_name: "DemoLib".to_string(),
                path: file.path.clone(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone(),
                symbol_id: format!("repo:{}:symbol:PI", context.repository.id),
                module_id: Some(module_id),
                name: "PI".to_string(),
                qualified_name: "DemoLib.PI".to_string(),
                kind: RepoSymbolKind::Type,
                path: file.path.clone(),
                line_start: Some(1),
                line_end: Some(1),
                signature: Some("model PI".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            imports: Vec::new(),
            examples: Vec::new(),
            docs: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &std::path::Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        Ok(RepositoryAnalysisOutput {
            repository: Some(RepositoryRecord {
                repo_id: context.repository.id.clone(),
                name: "DemoLib".to_string(),
                path: repository_root.display().to_string(),
                url: None,
                revision: None,
                version: None,
                uuid: None,
                dependencies: Vec::new(),
            }),
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone(),
                module_id: format!("repo:{}:module:DemoLib", context.repository.id),
                qualified_name: "DemoLib".to_string(),
                path: "PI.mo".to_string(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone(),
                symbol_id: format!("repo:{}:symbol:PI", context.repository.id),
                module_id: Some(format!("repo:{}:module:DemoLib", context.repository.id)),
                name: "PI".to_string(),
                qualified_name: "DemoLib.PI".to_string(),
                kind: RepoSymbolKind::Type,
                path: "PI.mo".to_string(),
                line_start: Some(1),
                line_end: Some(1),
                signature: Some("model PI".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            ..RepositoryAnalysisOutput::default()
        })
    }
}

pub(super) fn bootstrap_builtin_registry_with_runtime_rust_plugin() -> Arc<PluginRegistry> {
    let mut registry =
        bootstrap_builtin_registry().unwrap_or_else(|error| panic!("bootstrap registry: {error}"));
    registry
        .register(RuntimeRustPlugin)
        .unwrap_or_else(|error| panic!("register Rust runtime plugin: {error}"));
    Arc::new(registry)
}
