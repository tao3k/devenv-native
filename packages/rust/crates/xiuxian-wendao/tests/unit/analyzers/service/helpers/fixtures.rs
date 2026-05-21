use std::collections::BTreeMap;

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{
    DocRecord, ExampleRecord, ImportKind, ImportRecord, ModuleRecord, RelationKind, RelationRecord,
    RepoSymbolKind, RepositoryRecord, SymbolRecord,
};

pub(crate) fn some_or_panic<T>(value: Option<T>, context: &str) -> T {
    value.unwrap_or_else(|| panic!("{context}"))
}

pub(crate) fn repository_record(repo_id: &str) -> RepositoryRecord {
    RepositoryRecord {
        repo_id: repo_id.to_string().into(),
        name: repo_id.to_string(),
        ..RepositoryRecord::default()
    }
}

pub(crate) fn module_record(
    repo_id: &str,
    module_id: &str,
    qualified_name: &str,
    path: &str,
) -> ModuleRecord {
    ModuleRecord {
        repo_id: repo_id.to_string().into(),
        module_id: module_id.to_string().into(),
        qualified_name: qualified_name.to_string(),
        path: path.to_string().into(),
    }
}

pub(crate) fn symbol_record(
    repo_id: &str,
    symbol_id: &str,
    module_id: Option<&str>,
    name: &str,
    qualified_name: &str,
    path: &str,
) -> SymbolRecord {
    SymbolRecord {
        repo_id: repo_id.to_string().into(),
        symbol_id: symbol_id.to_string().into(),
        module_id: module_id.map(str::to_string).map(Into::into),
        name: name.to_string(),
        qualified_name: qualified_name.to_string(),
        kind: RepoSymbolKind::Function,
        path: path.to_string().into(),
        line_start: None,
        line_end: None,
        signature: Some(format!("fn {name}()")),
        audit_status: None,
        verification_state: None,
        attributes: BTreeMap::new(),
    }
}

pub(crate) fn doc_record(repo_id: &str, doc_id: &str, title: &str, path: &str) -> DocRecord {
    DocRecord {
        repo_id: repo_id.to_string().into(),
        doc_id: doc_id.to_string().into(),
        title: title.to_string(),
        path: path.to_string().into(),
        format: Some("markdown".to_string()),
        doc_target: None,
    }
}

pub(crate) fn example_record(
    repo_id: &str,
    example_id: &str,
    title: &str,
    path: &str,
) -> ExampleRecord {
    ExampleRecord {
        repo_id: repo_id.to_string().into(),
        example_id: example_id.to_string().into(),
        title: title.to_string(),
        path: path.to_string().into(),
        summary: Some(format!("{title} summary")),
    }
}

pub(crate) fn import_record(
    repo_id: &str,
    module_id: &str,
    import_name: &str,
    target_package: &str,
    source_module: &str,
) -> ImportRecord {
    ImportRecord {
        repo_id: repo_id.to_string().into(),
        module_id: module_id.to_string().into(),
        path: String::new().into(),
        import_name: import_name.to_string(),
        target_package: target_package.to_string(),
        source_module: source_module.to_string(),
        kind: ImportKind::Module,
        line_start: None,
        resolved_id: None,
        attributes: BTreeMap::new(),
    }
}

pub(crate) fn analysis_fixture() -> RepositoryAnalysisOutput {
    let repo_id = "repo-a";
    RepositoryAnalysisOutput {
        repository: Some(repository_record(repo_id)),
        modules: vec![
            module_record(repo_id, "mod-a", "alpha.beta", "src/alpha/beta.rs"),
            module_record(repo_id, "mod-b", "omega.gamma", "src/omega/gamma.rs"),
        ],
        symbols: vec![
            symbol_record(
                repo_id,
                "sym-a",
                Some("mod-a"),
                "Solve",
                "alpha.beta::Solve",
                "src/alpha/beta.rs#solve",
            ),
            symbol_record(
                repo_id,
                "sym-b",
                Some("mod-b"),
                "Helper",
                "omega.gamma::Helper",
                "src/omega/gamma.rs#helper",
            ),
        ],
        imports: vec![import_record(
            repo_id,
            "mod-a",
            "solver",
            "sciml-solver",
            "alpha.beta",
        )],
        examples: vec![
            example_record(repo_id, "ex-a", "Solve Example", "examples/solve.rs"),
            example_record(repo_id, "ex-b", "Helper Example", "examples/helper.rs"),
        ],
        docs: vec![
            doc_record(repo_id, "doc-a", "Alpha Guide", "docs/alpha.md"),
            doc_record(repo_id, "doc-b", "Symbol Guide", "docs/symbol.md"),
        ],
        relations: vec![
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: "doc-a".to_string(),
                target_id: "mod-a".to_string(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: "doc-b".to_string(),
                target_id: "sym-a".to_string(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: "doc-b".to_string(),
                target_id: "sym-a".to_string(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: "ex-a".to_string(),
                target_id: "sym-a".to_string(),
                kind: RelationKind::ExampleOf,
            },
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: "ex-a".to_string(),
                target_id: "mod-a".to_string(),
                kind: RelationKind::ExampleOf,
            },
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: "ex-a".to_string(),
                target_id: "sym-a".to_string(),
                kind: RelationKind::ExampleOf,
            },
        ],
        diagnostics: Vec::new(),
    }
}
