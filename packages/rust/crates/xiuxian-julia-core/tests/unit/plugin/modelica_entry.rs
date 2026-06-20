type TestResult = Result<(), Box<dyn std::error::Error>>;

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, RegisteredRepository, RepoIntelligencePlugin, RepoSourceFile,
    RepositoryPluginConfig,
};

use super::ModelicaRepoIntelligencePlugin;
use crate::julia_plugin_test_support::common::{
    ensure_linked_modelica_parser_summary_service, repo_root,
    skip_linked_modelica_parser_summary_service_if_unavailable,
};

include!("modelica_entry/live_analysis.rs");
include!("modelica_entry/context_resolution.rs");

fn analysis_context(repo_id: &str, repository_root: &Path) -> AnalysisContext {
    AnalysisContext {
        repository: RegisteredRepository {
            id: repo_id.to_string(),
            plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
            ..RegisteredRepository::default()
        },
        repository_root: repository_root.to_path_buf(),
    }
}
