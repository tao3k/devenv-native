//! Integration snapshot for projected pages.

use insta::assert_json_snapshot;
use std::collections::BTreeMap;
use xiuxian_wendao::analyzers::{
    DocRecord, ExampleRecord, ModuleRecord, RelationKind, RelationRecord, RepoProjectedPagesQuery,
    RepoSymbolKind, RepositoryAnalysisOutput, RepositoryRecord, SymbolRecord,
    build_repo_projected_pages,
};

#[test]
fn builds_projected_pages_from_stage_one_records() {
    let analysis = sample_projection_analysis_output();

    let result = build_repo_projected_pages(
        &RepoProjectedPagesQuery {
            repo_id: "demo".to_string(),
        },
        &analysis,
    );

    insta::with_settings!({
        snapshot_path => "../snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_json_snapshot!(
            "repo_projected_pages__repo_projected_pages",
            result
        );
    });
}

fn sample_projection_analysis_output() -> RepositoryAnalysisOutput {
    RepositoryAnalysisOutput {
        repository: Some(RepositoryRecord {
            repo_id: "demo".to_string().into(),
            name: "Demo".to_string(),
            path: "/tmp/demo".to_string().into(),
            url: None,
            revision: Some("abc123".to_string()),
            version: None,
            uuid: None,
            dependencies: Vec::new(),
        }),
        modules: vec![ModuleRecord {
            repo_id: "demo".to_string().into(),
            module_id: "repo:demo:module:Demo.Controllers".to_string().into(),
            qualified_name: "Demo.Controllers".to_string(),
            path: "Controllers/package.mo".to_string().into(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: "demo".to_string().into(),
            symbol_id: "repo:demo:symbol:Demo.Controllers.PI".to_string().into(),
            module_id: Some("repo:demo:module:Demo.Controllers".to_string().into()),
            name: "PI".to_string(),
            qualified_name: "Demo.Controllers.PI".to_string(),
            kind: RepoSymbolKind::Type,
            path: "Controllers/PI.mo".to_string().into(),
            line_start: None,
            line_end: None,
            signature: None,
            audit_status: None,
            verification_state: None,
            attributes: BTreeMap::new(),
        }],
        imports: Vec::new(),
        examples: vec![ExampleRecord {
            repo_id: "demo".to_string().into(),
            example_id: "repo:demo:example:Controllers/Examples/Step.mo"
                .to_string()
                .into(),
            title: "Step".to_string(),
            path: "Controllers/Examples/Step.mo".to_string().into(),
            summary: None,
        }],
        docs: vec![
            DocRecord {
                repo_id: "demo".to_string().into(),
                doc_id: "repo:demo:doc:Controllers/UsersGuide/Tutorial/FirstSteps.mo"
                    .to_string()
                    .into(),
                title: "First Steps".to_string(),
                path: "Controllers/UsersGuide/Tutorial/FirstSteps.mo"
                    .to_string()
                    .into(),
                format: Some("modelica_users_guide_tutorial".to_string()),
                doc_target: None,
            },
            DocRecord {
                repo_id: "demo".to_string().into(),
                doc_id: "repo:demo:doc:Controllers/PI.mo#annotation.documentation"
                    .to_string()
                    .into(),
                title: "PI documentation".to_string(),
                path: "Controllers/PI.mo#annotation.documentation"
                    .to_string()
                    .into(),
                format: Some("modelica_annotation".to_string()),
                doc_target: None,
            },
        ],
        relations: vec![
            RelationRecord {
                repo_id: "demo".to_string().into(),
                source_id: "repo:demo:doc:Controllers/UsersGuide/Tutorial/FirstSteps.mo"
                    .to_string(),
                target_id: "repo:demo:module:Demo.Controllers".to_string(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: "demo".to_string().into(),
                source_id: "repo:demo:doc:Controllers/PI.mo#annotation.documentation".to_string(),
                target_id: "repo:demo:symbol:Demo.Controllers.PI".to_string(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: "demo".to_string().into(),
                source_id: "repo:demo:example:Controllers/Examples/Step.mo".to_string(),
                target_id: "repo:demo:module:Demo.Controllers".to_string(),
                kind: RelationKind::ExampleOf,
            },
        ],
        diagnostics: Vec::new(),
    }
}
