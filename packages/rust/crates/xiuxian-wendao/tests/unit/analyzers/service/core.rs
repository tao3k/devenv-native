use std::path::Path;

use crate::analyzers::DocRecord;
use crate::analyzers::PluginRegistry;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::RepositoryRecord;
use crate::analyzers::service::analysis::analyze_registered_repository_bundle_with_registry;
use crate::analyzers::service::merge::{
    hydrate_repository_record, merge_repository_analysis, merge_repository_record,
};
use crate::analyzers::{RefineEntityDocRequest, RefineEntityDocResponse};
use crate::analyzers::{RegisteredRepository, RepositoryRefreshPolicy};
use crate::analyzers::{RepoIntelligenceError, RepositoryPluginConfig};
use xiuxian_git_repo::LocalCheckoutMetadata;

#[test]
fn test_refine_contract_serialization() {
    let req = RefineEntityDocRequest {
        repo_id: "test".to_string(),
        entity_id: "sym1".to_string(),
        user_hints: Some("more details".to_string()),
    };
    let res = RefineEntityDocResponse {
        repo_id: "test".to_string(),
        entity_id: "sym1".to_string(),
        refined_content: "Refined".to_string(),
        verification_state: "verified".to_string(),
    };
    assert_eq!(req.repo_id, "test");
    assert_eq!(res.verification_state, "verified");
}

#[test]
fn merge_repository_record_keeps_first_authority_and_fills_missing_metadata() {
    let base = RepositoryRecord {
        repo_id: "demo".to_string().into(),
        name: "demo".to_string(),
        path: "/tmp/demo".to_string().into(),
        url: Some("https://base.invalid/demo.git".to_string()),
        revision: Some("base-rev".to_string()),
        version: None,
        uuid: None,
        dependencies: Vec::new(),
    };
    let overlay = RepositoryRecord {
        repo_id: "demo".to_string().into(),
        name: "DemoPkg".to_string(),
        path: "/tmp/demo".to_string().into(),
        url: None,
        revision: None,
        version: Some("0.1.0".to_string()),
        uuid: Some("uuid-demo".to_string()),
        dependencies: vec!["LinearAlgebra".to_string()],
    };

    let merged = merge_repository_record(base, overlay);

    assert_eq!(merged.name, "demo");
    assert_eq!(merged.url.as_deref(), Some("https://base.invalid/demo.git"));
    assert_eq!(merged.revision.as_deref(), Some("base-rev"));
    assert_eq!(merged.version.as_deref(), Some("0.1.0"));
    assert_eq!(merged.uuid.as_deref(), Some("uuid-demo"));
    assert_eq!(merged.dependencies, vec!["LinearAlgebra".to_string()]);
}

#[test]
fn merge_repository_analysis_keeps_first_doc_for_duplicate_doc_ids() {
    let doc_id = "repo:sample:doc:README.md";
    let mut base = RepositoryAnalysisOutput {
        docs: vec![test_doc_record("sample", doc_id, "README", "README.md")],
        ..RepositoryAnalysisOutput::default()
    };
    let overlay = RepositoryAnalysisOutput {
        docs: vec![
            test_doc_record("sample", doc_id, "Projectionica", "README.md"),
            test_doc_record(
                "sample",
                "repo:sample:doc:docs/guide.md",
                "Guide",
                "docs/guide.md",
            ),
        ],
        ..RepositoryAnalysisOutput::default()
    };

    merge_repository_analysis(&mut base, overlay);

    let titles = base
        .docs
        .iter()
        .map(|doc| doc.title.as_str())
        .collect::<Vec<_>>();
    assert_eq!(titles, vec!["README", "Guide"]);
}

fn test_doc_record(repo_id: &str, doc_id: &str, title: &str, path: &str) -> DocRecord {
    DocRecord {
        repo_id: repo_id.to_string().into(),
        doc_id: doc_id.to_string().into(),
        title: title.to_string(),
        path: path.to_string().into(),
        format: Some("md".to_string()),
        doc_target: None,
    }
}

#[test]
fn hydrate_repository_record_backfills_checkout_metadata() {
    let repository = RegisteredRepository {
        id: "sample".to_string(),
        path: Some("/tmp/sample".into()),
        url: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: Vec::new(),
    };
    let mut record = RepositoryRecord {
        repo_id: String::new().into(),
        name: String::new(),
        path: String::new().into(),
        url: None,
        revision: None,
        version: None,
        uuid: None,
        dependencies: Vec::new(),
    };

    hydrate_repository_record(
        &mut record,
        &repository,
        Path::new("/tmp/sample"),
        Some(&LocalCheckoutMetadata {
            revision: Some("abc123".to_string()),
            remote_url: Some("https://example.invalid/sample.git".to_string()),
        }),
    );

    assert_eq!(record.repo_id, "sample");
    assert_eq!(record.name, "sample");
    assert_eq!(record.path, "/tmp/sample");
    assert_eq!(
        record.url.as_deref(),
        Some("https://example.invalid/sample.git")
    );
    assert_eq!(record.revision.as_deref(), Some("abc123"));
}

#[test]
fn analyze_registered_repository_bundle_requires_repo_intelligence_plugins() {
    let repository = RegisteredRepository {
        id: "sample".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("repo-content".to_string())],
        ..RegisteredRepository::default()
    };
    let registry = PluginRegistry::new();

    let Err(error) =
        analyze_registered_repository_bundle_with_registry(&repository, Path::new("."), &registry)
    else {
        panic!("search-only repositories should require a repo intelligence plugin");
    };

    assert!(matches!(
        error,
        RepoIntelligenceError::MissingRepoIntelligencePlugins { repo_id }
            if repo_id == "sample"
    ));
}
