use std::collections::BTreeMap;

use super::{
    JuliaAnalyzedFile, JuliaParserDocAttachment, JuliaParserDocTargetKind, JuliaParserFileSummary,
    JuliaParserSymbol, JuliaParserSymbolKind, RelationKind, RepoSymbolKind,
    build_docstring_records, build_docstring_relations, build_symbol_records,
    collect_symbol_records,
};

#[test]
fn build_symbol_records_preserves_same_file_overloads_without_export_placeholder() {
    let records = build_symbol_records(
        "demo",
        "src/SameFile.jl",
        "Demo",
        &["solve".to_string()],
        &[
            JuliaParserSymbol {
                name: "solve".to_string(),
                kind: JuliaParserSymbolKind::Function,
                signature: Some("solve(problem::Problem)".to_string()),
                line_start: Some(10),
                line_end: Some(12),
                attributes: BTreeMap::new(),
            },
            JuliaParserSymbol {
                name: "solve".to_string(),
                kind: JuliaParserSymbolKind::Function,
                signature: Some("solve(problem::Problem, dt::Float64)".to_string()),
                line_start: Some(14),
                line_end: Some(18),
                attributes: BTreeMap::new(),
            },
        ],
    );

    assert_eq!(records.len(), 2);
    assert!(
        records
            .iter()
            .all(|record| record.kind == RepoSymbolKind::Function)
    );
    assert!(
        records
            .iter()
            .all(|record| record.symbol_id.starts_with("repo:demo:symbol:Demo.solve@"))
    );
    assert_ne!(records[0].symbol_id, records[1].symbol_id);
}

#[test]
fn collect_symbol_records_preserves_cross_file_overloads_without_export_placeholder() {
    let records = collect_symbol_records(
        "demo",
        "Demo",
        &[
            JuliaAnalyzedFile {
                path: "src/A.jl".to_string(),
                summary: JuliaParserFileSummary {
                    module_name: Some("Demo".to_string()),
                    exports: vec!["solve".to_string()],
                    imports: Vec::new(),
                    symbols: vec![JuliaParserSymbol {
                        name: "solve".to_string(),
                        kind: JuliaParserSymbolKind::Function,
                        signature: Some("solve(problem::Problem)".to_string()),
                        line_start: Some(10),
                        line_end: Some(12),
                        attributes: BTreeMap::new(),
                    }],
                    docstrings: Vec::new(),
                    includes: Vec::new(),
                },
            },
            JuliaAnalyzedFile {
                path: "src/B.jl".to_string(),
                summary: JuliaParserFileSummary {
                    module_name: Some("Demo".to_string()),
                    exports: Vec::new(),
                    imports: Vec::new(),
                    symbols: vec![JuliaParserSymbol {
                        name: "solve".to_string(),
                        kind: JuliaParserSymbolKind::Function,
                        signature: Some("solve(problem::Problem, dt::Float64)".to_string()),
                        line_start: Some(20),
                        line_end: Some(24),
                        attributes: BTreeMap::new(),
                    }],
                    docstrings: Vec::new(),
                    includes: Vec::new(),
                },
            },
        ],
    );

    assert_eq!(records.len(), 2);
    assert!(
        records
            .iter()
            .all(|record| record.kind == RepoSymbolKind::Function)
    );
    assert!(
        records
            .iter()
            .all(|record| record.symbol_id.starts_with("repo:demo:symbol:Demo.solve@"))
    );
    assert_ne!(records[0].symbol_id, records[1].symbol_id);
}

#[test]
fn build_docstring_records_and_relations_resolve_overloaded_targets_by_parser_lines() {
    let symbols = build_symbol_records(
        "demo",
        "src/SameFile.jl",
        "Demo",
        &[],
        &[
            JuliaParserSymbol {
                name: "solve".to_string(),
                kind: JuliaParserSymbolKind::Function,
                signature: Some("solve(problem::Problem)".to_string()),
                line_start: Some(10),
                line_end: Some(12),
                attributes: BTreeMap::from([("owner_path".to_string(), "Demo".to_string())]),
            },
            JuliaParserSymbol {
                name: "solve".to_string(),
                kind: JuliaParserSymbolKind::Function,
                signature: Some("solve(problem::Problem, dt::Float64)".to_string()),
                line_start: Some(20),
                line_end: Some(24),
                attributes: BTreeMap::from([("owner_path".to_string(), "Demo".to_string())]),
            },
        ],
    );

    let docstrings = vec![
        JuliaParserDocAttachment {
            target_name: "solve".to_string(),
            target_kind: JuliaParserDocTargetKind::Symbol,
            target_path: Some("Demo.solve".to_string()),
            target_line_start: Some(10),
            target_line_end: Some(12),
            content: "Solve the main problem.".to_string(),
        },
        JuliaParserDocAttachment {
            target_name: "solve".to_string(),
            target_kind: JuliaParserDocTargetKind::Symbol,
            target_path: Some("Demo.solve".to_string()),
            target_line_start: Some(20),
            target_line_end: Some(24),
            content: "Solve with an explicit timestep.".to_string(),
        },
    ];

    let docs = build_docstring_records("demo", "src/SameFile.jl", "Demo", &symbols, &docstrings);
    let relations = build_docstring_relations(
        "demo",
        "repo:demo:module:Demo",
        "Demo",
        &symbols,
        &docstrings,
        "src/SameFile.jl",
    );

    assert_eq!(docs.len(), 2);
    assert_eq!(relations.len(), 2);
    assert!(docs.iter().all(|doc| {
        doc.doc_id
            .contains("#symbol-id:repo:demo:symbol:Demo.solve@")
    }));
    assert_eq!(
        docs[0]
            .doc_target
            .as_ref()
            .map(|target| target.path.as_deref()),
        Some(Some("Demo.solve"))
    );
    assert_eq!(
        docs[1].doc_target.as_ref().map(|target| target.line_start),
        Some(Some(20))
    );
    assert_ne!(docs[0].doc_id, docs[1].doc_id);
    assert!(
        relations
            .iter()
            .all(|relation| relation.kind == RelationKind::Documents)
    );
    assert_ne!(relations[0].target_id, relations[1].target_id);
}
