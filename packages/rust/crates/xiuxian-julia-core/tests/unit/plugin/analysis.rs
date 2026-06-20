type TestResult = Result<(), Box<dyn std::error::Error>>;

use std::fs;
use std::path::Path;

use serde_json::json;
use tempfile::TempDir;
use xiuxian_wendao_core::repo_intelligence::RepositoryPluginConfig;
use xiuxian_wendao_core::repo_intelligence::{AnalysisContext, RegisteredRepository};

use super::{
    analyze_repository, load_modelica_repository_context_for_source, preflight_repository,
};
use crate::julia_plugin_test_support::common::{
    assert_sorted_json_snapshot, ensure_linked_modelica_parser_summary_service,
    skip_linked_modelica_parser_summary_service_if_unavailable,
};

include!("analysis/repository_analysis.rs");
include!("analysis/repository_context.rs");

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

fn write_modelica_file(path: &Path, contents: &str) -> TestResult {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}
