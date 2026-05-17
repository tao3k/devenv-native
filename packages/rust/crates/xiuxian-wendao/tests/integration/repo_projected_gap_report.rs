//! Integration snapshot for deterministic projected deep-wiki gap reporting.

use std::collections::BTreeMap;

use xiuxian_wendao::analyzers::{
    DocRecord, ExampleRecord, ModuleRecord, ProjectedGapKind, RelationKind, RelationRecord,
    RepoProjectedGapReportQuery, RepoSymbolKind, RepositoryAnalysisOutput, RepositoryRecord,
    SymbolRecord, build_repo_projected_gap_report,
};

#[test]
fn builds_projected_gap_report_from_stage_two_projection_quality_signals() {
    let repo_id = "demo";
    let analysis = sample_projected_gap_report_analysis(repo_id);

    let result = build_repo_projected_gap_report(
        &RepoProjectedGapReportQuery {
            repo_id: repo_id.to_string(),
        },
        &analysis,
    );

    assert_eq!(
        result.summary.gap_count,
        result.gaps.len(),
        "gap summary should match materialized gap count"
    );
    assert!(
        result
            .gaps
            .iter()
            .any(|gap| gap.kind == ProjectedGapKind::SymbolReferenceUnverified),
        "expected an unverified symbol-reference gap"
    );

    assert_repo_json_snapshot("repo_projected_gap_report_result", result);
}

fn sample_projected_gap_report_analysis(repo_id: &str) -> RepositoryAnalysisOutput {
    let controllers_module_id = format!("repo:{repo_id}:module:Demo.Controllers");
    let utilities_module_id = format!("repo:{repo_id}:module:Demo.Utilities");
    let solve_symbol_id = format!("repo:{repo_id}:symbol:Demo.Controllers.solve");
    let drift_symbol_id = format!("repo:{repo_id}:symbol:Demo.Controllers.drift");
    let helper_symbol_id = format!("repo:{repo_id}:symbol:Demo.Utilities.Helper");
    let detached_example_id = format!("repo:{repo_id}:example:examples/detached_demo.jl");
    let first_steps_doc_id = format!("repo:{repo_id}:doc:docs/first_steps.md");
    let solve_doc_id = format!("repo:{repo_id}:doc:src/Demo.jl#symbol:solve");
    let drift_doc_id = format!("repo:{repo_id}:doc:src/Demo.jl#symbol:drift");
    let orphan_doc_id = format!("repo:{repo_id}:doc:docs/users_guide/concepts.md");

    RepositoryAnalysisOutput {
        repository: Some(sample_repository_record(repo_id)),
        modules: sample_modules(repo_id, &controllers_module_id, &utilities_module_id),
        symbols: sample_symbols(
            repo_id,
            &controllers_module_id,
            &utilities_module_id,
            &solve_symbol_id,
            &drift_symbol_id,
            &helper_symbol_id,
        ),
        imports: Vec::new(),
        examples: sample_examples(repo_id, &detached_example_id),
        docs: sample_docs(
            repo_id,
            &first_steps_doc_id,
            &solve_doc_id,
            &drift_doc_id,
            &orphan_doc_id,
        ),
        relations: sample_relations(
            repo_id,
            &first_steps_doc_id,
            &solve_doc_id,
            &drift_doc_id,
            &controllers_module_id,
            &solve_symbol_id,
            &drift_symbol_id,
        ),
        diagnostics: Vec::new(),
    }
}

fn sample_repository_record(repo_id: &str) -> RepositoryRecord {
    RepositoryRecord {
        repo_id: repo_id.to_string().into(),
        name: "Demo".to_string(),
        path: "/tmp/demo".to_string().into(),
        url: None,
        revision: Some("fixture".to_string()),
        version: None,
        uuid: None,
        dependencies: Vec::new(),
    }
}

fn sample_modules(
    repo_id: &str,
    controllers_module_id: &str,
    utilities_module_id: &str,
) -> Vec<ModuleRecord> {
    vec![
        ModuleRecord {
            repo_id: repo_id.to_string().into(),
            module_id: controllers_module_id.to_string().into(),
            qualified_name: "Demo.Controllers".to_string(),
            path: "src/Demo.jl".to_string().into(),
        },
        ModuleRecord {
            repo_id: repo_id.to_string().into(),
            module_id: utilities_module_id.to_string().into(),
            qualified_name: "Demo.Utilities".to_string(),
            path: "src/Utilities.jl".to_string().into(),
        },
    ]
}

fn sample_symbols(
    repo_id: &str,
    controllers_module_id: &str,
    utilities_module_id: &str,
    solve_symbol_id: &str,
    drift_symbol_id: &str,
    helper_symbol_id: &str,
) -> Vec<SymbolRecord> {
    vec![
        SymbolRecord {
            repo_id: repo_id.to_string().into(),
            symbol_id: solve_symbol_id.to_string().into(),
            module_id: Some(controllers_module_id.to_string().into()),
            name: "solve".to_string(),
            qualified_name: "Demo.Controllers.solve".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/Demo.jl".to_string().into(),
            line_start: None,
            line_end: None,
            signature: Some("solve()".to_string()),
            audit_status: None,
            verification_state: Some("verified".to_string().into()),
            attributes: BTreeMap::new(),
        },
        SymbolRecord {
            repo_id: repo_id.to_string().into(),
            symbol_id: drift_symbol_id.to_string().into(),
            module_id: Some(controllers_module_id.to_string().into()),
            name: "drift".to_string(),
            qualified_name: "Demo.Controllers.drift".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/Demo.jl".to_string().into(),
            line_start: None,
            line_end: None,
            signature: Some("drift()".to_string()),
            audit_status: None,
            verification_state: Some("unverified".to_string().into()),
            attributes: BTreeMap::new(),
        },
        SymbolRecord {
            repo_id: repo_id.to_string().into(),
            symbol_id: helper_symbol_id.to_string().into(),
            module_id: Some(utilities_module_id.to_string().into()),
            name: "Helper".to_string(),
            qualified_name: "Demo.Utilities.Helper".to_string(),
            kind: RepoSymbolKind::Type,
            path: "src/Utilities.jl".to_string().into(),
            line_start: None,
            line_end: None,
            signature: Some("struct Helper".to_string()),
            audit_status: None,
            verification_state: Some("unknown".to_string().into()),
            attributes: BTreeMap::new(),
        },
    ]
}

fn sample_examples(repo_id: &str, detached_example_id: &str) -> Vec<ExampleRecord> {
    vec![
        ExampleRecord {
            repo_id: repo_id.to_string().into(),
            example_id: format!("repo:{repo_id}:example:examples/solve_demo.jl").into(),
            title: "Solve demo".to_string(),
            path: "examples/solve_demo.jl".to_string().into(),
            summary: None,
        },
        ExampleRecord {
            repo_id: repo_id.to_string().into(),
            example_id: detached_example_id.to_string().into(),
            title: "Detached demo".to_string(),
            path: "examples/detached_demo.jl".to_string().into(),
            summary: None,
        },
    ]
}

fn sample_docs(
    repo_id: &str,
    first_steps_doc_id: &str,
    solve_doc_id: &str,
    drift_doc_id: &str,
    orphan_doc_id: &str,
) -> Vec<DocRecord> {
    vec![
        DocRecord {
            repo_id: repo_id.to_string().into(),
            doc_id: first_steps_doc_id.to_string().into(),
            title: "First Steps".to_string(),
            path: "docs/first_steps.md".to_string().into(),
            format: Some("tutorial".to_string()),
            doc_target: None,
        },
        DocRecord {
            repo_id: repo_id.to_string().into(),
            doc_id: solve_doc_id.to_string().into(),
            title: "solve".to_string(),
            path: "src/Demo.jl#symbol:solve".to_string().into(),
            format: Some("api".to_string()),
            doc_target: None,
        },
        DocRecord {
            repo_id: repo_id.to_string().into(),
            doc_id: drift_doc_id.to_string().into(),
            title: "General Notes".to_string(),
            path: "src/Demo.jl#symbol:drift".to_string().into(),
            format: Some("api".to_string()),
            doc_target: None,
        },
        DocRecord {
            repo_id: repo_id.to_string().into(),
            doc_id: orphan_doc_id.to_string().into(),
            title: "Concepts".to_string(),
            path: "docs/users_guide/concepts.md".to_string().into(),
            format: Some("guide".to_string()),
            doc_target: None,
        },
    ]
}

fn sample_relations(
    repo_id: &str,
    first_steps_doc_id: &str,
    solve_doc_id: &str,
    drift_doc_id: &str,
    controllers_module_id: &str,
    solve_symbol_id: &str,
    drift_symbol_id: &str,
) -> Vec<RelationRecord> {
    vec![
        RelationRecord {
            repo_id: repo_id.to_string().into(),
            source_id: first_steps_doc_id.to_string(),
            target_id: controllers_module_id.to_string(),
            kind: RelationKind::Documents,
        },
        RelationRecord {
            repo_id: repo_id.to_string().into(),
            source_id: solve_doc_id.to_string(),
            target_id: solve_symbol_id.to_string(),
            kind: RelationKind::Documents,
        },
        RelationRecord {
            repo_id: repo_id.to_string().into(),
            source_id: drift_doc_id.to_string(),
            target_id: drift_symbol_id.to_string(),
            kind: RelationKind::Documents,
        },
        RelationRecord {
            repo_id: repo_id.to_string().into(),
            source_id: format!("repo:{repo_id}:example:examples/solve_demo.jl"),
            target_id: controllers_module_id.to_string(),
            kind: RelationKind::ExampleOf,
        },
    ]
}

fn assert_repo_json_snapshot(name: &str, value: impl serde::Serialize) {
    insta::with_settings!({
        snapshot_path => "../snapshots/repo_intelligence",
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}
