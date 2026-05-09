//! Integration tests for Repo Intelligence plugin registry behavior.

use std::fs;

use crate::support::repo_fixture;
use xiuxian_wendao::analyzers::{
    AnalysisContext, PluginAnalysisOutput, PluginLinkContext, PluginRegistry, RegisteredRepository,
    RelationRecord, RepoIntelligenceError, RepoIntelligencePlugin, RepoOverviewQuery,
    RepoSourceFile, RepositoryAnalysisOutput, RepositoryPluginConfig, RepositoryRecord,
    RepositoryRef, RepositoryRefreshPolicy, repo_overview_from_config_with_registry,
};

#[derive(Debug)]
struct TestPlugin {
    plugin_id: &'static str,
    supported_repo: &'static str,
}

impl RepoIntelligencePlugin for TestPlugin {
    fn id(&self) -> &'static str {
        self.plugin_id
    }

    fn supports_repository(&self, repository: &RegisteredRepository) -> bool {
        repository.id == self.supported_repo
    }

    fn analyze_file(
        &self,
        _context: &AnalysisContext,
        _file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        Ok(PluginAnalysisOutput::default())
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        _repository_root: &std::path::Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        Ok(RepositoryAnalysisOutput {
            repository: Some(self.repository_record(&context.repository)),
            ..RepositoryAnalysisOutput::default()
        })
    }

    fn enrich_relations(
        &self,
        _context: &PluginLinkContext,
    ) -> Result<Vec<RelationRecord>, RepoIntelligenceError> {
        Ok(Vec::new())
    }
}

impl TestPlugin {
    fn repository_record(&self, repository: &RegisteredRepository) -> RepositoryRecord {
        RepositoryRecord {
            repo_id: repository.id.clone(),
            name: format!("{}-{}", self.plugin_id, repository.id),
            path: repository
                .path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
            url: repository.url.clone(),
            revision: None,
            version: None,
            uuid: None,
            dependencies: Vec::new(),
        }
    }
}

fn sample_repository() -> RegisteredRepository {
    RegisteredRepository {
        id: "sciml-diffeq".to_string(),
        path: None,
        url: Some("https://github.com/SciML/DifferentialEquations.jl.git".to_string()),
        git_ref: Some(RepositoryRef::Branch("main".to_string())),
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
    }
}

#[test]
fn register_rejects_duplicate_plugin_ids() {
    let mut registry = PluginRegistry::new();

    let Ok(()) = registry.register(TestPlugin {
        plugin_id: "julia-code-parser",
        supported_repo: "sciml-diffeq",
    }) else {
        panic!("first registration should succeed");
    };

    let Err(error) = registry.register(TestPlugin {
        plugin_id: "julia-code-parser",
        supported_repo: "sciml-diffeq",
    }) else {
        panic!("duplicate registration should fail");
    };

    assert_eq!(
        error,
        RepoIntelligenceError::DuplicatePlugin {
            plugin_id: "julia-code-parser".to_string(),
        }
    );
}

#[test]
fn resolve_for_repository_returns_matching_plugins() {
    let mut registry = PluginRegistry::new();
    let Ok(()) = registry.register(TestPlugin {
        plugin_id: "julia-code-parser",
        supported_repo: "sciml-diffeq",
    }) else {
        panic!("registration should succeed");
    };

    let Ok(resolved) = registry.resolve_for_repository(&sample_repository()) else {
        panic!("plugin resolution should succeed");
    };

    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].id(), "julia");
}

#[test]
fn resolve_for_repository_fails_when_plugin_is_missing() {
    let registry = PluginRegistry::new();

    let Err(error) = registry.resolve_for_repository(&sample_repository()) else {
        panic!("missing plugin should fail");
    };

    assert_eq!(
        error,
        RepoIntelligenceError::MissingPlugin {
            plugin_id: "julia-code-parser".to_string(),
        }
    );
}

#[test]
fn custom_registry_drives_repo_overview_queries() -> repo_fixture::TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = repo_fixture::create_sample_julia_repo(temp.path(), "ExternalPkg", true)?;
    let config_path = temp.path().join("external.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.projects.external-sample]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let mut registry = PluginRegistry::new();
    registry.register(TestPlugin {
        plugin_id: "modelica",
        supported_repo: "external-sample",
    })?;

    let overview = repo_overview_from_config_with_registry(
        &RepoOverviewQuery {
            repo_id: "external-sample".to_string(),
        },
        Some(&config_path),
        temp.path(),
        &registry,
    )?;

    assert_eq!(overview.repo_id, "external-sample");
    assert_eq!(overview.display_name, "modelica-external-sample");
    Ok(())
}
