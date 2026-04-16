use std::collections::BTreeMap;

use crate::analyzers::cache::RepositoryAnalysisCacheKey;
use crate::analyzers::{
    DocRecord, ExampleRecord, ImportKind, ImportRecord, ModuleRecord, RelationKind, RelationRecord,
    RepoSymbolKind, RepositoryAnalysisOutput, RepositoryRecord, SymbolRecord,
};

struct SearchFixtureIds {
    module: String,
    solve_symbol: String,
    problem_symbol: String,
    readme_doc: String,
    solve_doc: String,
    problem_doc: String,
    example: String,
}

pub(super) fn sample_search_analysis(repo_id: &str) -> RepositoryAnalysisOutput {
    let ids = sample_search_fixture_ids(repo_id);

    RepositoryAnalysisOutput {
        repository: Some(sample_repository_record(repo_id)),
        modules: vec![sample_module_record(repo_id, ids.module.as_str())],
        symbols: sample_symbol_records(repo_id, &ids),
        imports: sample_import_records(repo_id, &ids),
        examples: sample_example_records(repo_id, &ids),
        docs: sample_doc_records(repo_id, &ids),
        relations: sample_relation_records(repo_id, &ids),
        diagnostics: Vec::new(),
    }
}

fn sample_search_fixture_ids(repo_id: &str) -> SearchFixtureIds {
    SearchFixtureIds {
        module: format!("repo:{repo_id}:module:ProjectionPkg"),
        solve_symbol: format!("repo:{repo_id}:symbol:ProjectionPkg.solve"),
        problem_symbol: format!("repo:{repo_id}:symbol:ProjectionPkg.Problem"),
        readme_doc: format!("repo:{repo_id}:doc:README.md"),
        solve_doc: format!("repo:{repo_id}:doc:src/ProjectionPkg.jl#symbol:solve"),
        problem_doc: format!("repo:{repo_id}:doc:src/ProjectionPkg.jl#symbol:Problem"),
        example: format!("repo:{repo_id}:example:examples/basic.jl"),
    }
}

fn sample_repository_record(repo_id: &str) -> RepositoryRecord {
    RepositoryRecord {
        repo_id: repo_id.to_string(),
        name: "ProjectionPkg".to_string(),
        path: format!("/virtual/repos/{repo_id}"),
        url: None,
        revision: Some("fixture".to_string()),
        version: Some("0.1.0".to_string()),
        uuid: None,
        dependencies: Vec::new(),
    }
}

fn sample_module_record(repo_id: &str, module_id: &str) -> ModuleRecord {
    ModuleRecord {
        repo_id: repo_id.to_string(),
        module_id: module_id.to_string(),
        qualified_name: "ProjectionPkg".to_string(),
        path: "src/ProjectionPkg.jl".to_string(),
    }
}

fn sample_symbol_records(repo_id: &str, ids: &SearchFixtureIds) -> Vec<SymbolRecord> {
    vec![
        SymbolRecord {
            repo_id: repo_id.to_string(),
            symbol_id: ids.solve_symbol.clone(),
            module_id: Some(ids.module.clone()),
            name: "solve".to_string(),
            qualified_name: "ProjectionPkg.solve".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/ProjectionPkg.jl".to_string(),
            line_start: None,
            line_end: None,
            signature: Some("solve(problem::Problem)".to_string()),
            audit_status: None,
            verification_state: None,
            attributes: BTreeMap::new(),
        },
        SymbolRecord {
            repo_id: repo_id.to_string(),
            symbol_id: ids.problem_symbol.clone(),
            module_id: Some(ids.module.clone()),
            name: "Problem".to_string(),
            qualified_name: "ProjectionPkg.Problem".to_string(),
            kind: RepoSymbolKind::Type,
            path: "src/ProjectionPkg.jl".to_string(),
            line_start: None,
            line_end: None,
            signature: Some("struct Problem".to_string()),
            audit_status: None,
            verification_state: None,
            attributes: BTreeMap::new(),
        },
    ]
}

fn sample_import_records(repo_id: &str, ids: &SearchFixtureIds) -> Vec<ImportRecord> {
    vec![ImportRecord {
        repo_id: repo_id.to_string(),
        module_id: ids.module.clone(),
        path: "src/ProjectionPkg.jl".to_string(),
        import_name: "solve".to_string(),
        target_package: "SciMLBase".to_string(),
        source_module: "BaseModelica".to_string(),
        kind: ImportKind::Symbol,
        line_start: None,
        resolved_id: Some(ids.solve_symbol.clone()),
        attributes: BTreeMap::new(),
    }]
}

fn sample_example_records(repo_id: &str, ids: &SearchFixtureIds) -> Vec<ExampleRecord> {
    vec![ExampleRecord {
        repo_id: repo_id.to_string(),
        example_id: ids.example.clone(),
        title: "basic".to_string(),
        path: "examples/basic.jl".to_string(),
        summary: Some("Solve a projection problem end to end.".to_string()),
    }]
}

fn sample_doc_records(repo_id: &str, ids: &SearchFixtureIds) -> Vec<DocRecord> {
    vec![
        DocRecord {
            repo_id: repo_id.to_string(),
            doc_id: ids.readme_doc.clone(),
            title: "README.md".to_string(),
            path: "README.md".to_string(),
            format: Some("md".to_string()),
            doc_target: None,
        },
        DocRecord {
            repo_id: repo_id.to_string(),
            doc_id: ids.problem_doc.clone(),
            title: "Problem".to_string(),
            path: "src/ProjectionPkg.jl#symbol:Problem".to_string(),
            format: Some("julia_docstring".to_string()),
            doc_target: None,
        },
        DocRecord {
            repo_id: repo_id.to_string(),
            doc_id: ids.solve_doc.clone(),
            title: "solve".to_string(),
            path: "src/ProjectionPkg.jl#symbol:solve".to_string(),
            format: Some("julia_docstring".to_string()),
            doc_target: None,
        },
    ]
}

fn sample_relation_records(repo_id: &str, ids: &SearchFixtureIds) -> Vec<RelationRecord> {
    vec![
        RelationRecord {
            repo_id: repo_id.to_string(),
            source_id: ids.readme_doc.clone(),
            target_id: ids.module.clone(),
            kind: RelationKind::Documents,
        },
        RelationRecord {
            repo_id: repo_id.to_string(),
            source_id: ids.problem_doc.clone(),
            target_id: ids.problem_symbol.clone(),
            kind: RelationKind::Documents,
        },
        RelationRecord {
            repo_id: repo_id.to_string(),
            source_id: ids.solve_doc.clone(),
            target_id: ids.solve_symbol.clone(),
            kind: RelationKind::Documents,
        },
        RelationRecord {
            repo_id: repo_id.to_string(),
            source_id: ids.example.clone(),
            target_id: ids.module.clone(),
            kind: RelationKind::ExampleOf,
        },
        RelationRecord {
            repo_id: repo_id.to_string(),
            source_id: ids.example.clone(),
            target_id: ids.solve_symbol.clone(),
            kind: RelationKind::ExampleOf,
        },
    ]
}

pub(super) fn sample_cache_key(repo_id: &str) -> RepositoryAnalysisCacheKey {
    RepositoryAnalysisCacheKey {
        repo_id: repo_id.to_string(),
        checkout_root: format!("/virtual/repos/{repo_id}"),
        analysis_identity: format!("analysis:{repo_id}"),
        checkout_revision: Some("fixture".to_string()),
        mirror_revision: Some("fixture".to_string()),
        tracking_revision: Some("fixture".to_string()),
        plugin_ids: vec!["fixture-plugin".to_string()],
    }
}

pub(super) fn ok_or_panic<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}
