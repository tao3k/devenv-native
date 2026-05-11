use std::collections::BTreeMap;

use crate::analyzers::{ExampleRecord, ModuleRecord, RepoSymbolKind, SymbolRecord};

use super::{ranked_example_matches, ranked_module_matches, ranked_symbol_matches};

fn module_record(module_id: &str, qualified_name: &str) -> ModuleRecord {
    ModuleRecord {
        repo_id: "repo".to_string().into(),
        module_id: module_id.to_string().into(),
        qualified_name: qualified_name.to_string(),
        path: format!("src/{qualified_name}.jl").into(),
    }
}

fn symbol_record(symbol_id: &str, name: &str, qualified_name: &str) -> SymbolRecord {
    SymbolRecord {
        repo_id: "repo".to_string().into(),
        symbol_id: symbol_id.to_string().into(),
        module_id: Some("repo:module:Sample".to_string().into()),
        name: name.to_string(),
        qualified_name: qualified_name.to_string(),
        kind: RepoSymbolKind::Function,
        path: "src/Sample.jl".to_string().into(),
        line_start: None,
        line_end: None,
        signature: None,
        audit_status: None,
        verification_state: None,
        attributes: BTreeMap::new(),
    }
}

fn example_record(example_id: &str, title: &str) -> ExampleRecord {
    ExampleRecord {
        repo_id: "repo".to_string().into(),
        example_id: example_id.to_string().into(),
        title: title.to_string(),
        path: format!("examples/{title}.jl").into(),
        summary: Some(format!("{title} walkthrough")),
    }
}

#[test]
fn ranked_module_matches_empty_query_keeps_input_order() {
    let modules = vec![
        module_record("repo:module:first", "First"),
        module_record("repo:module:second", "Second"),
    ];

    let ranked = ranked_module_matches("", &modules, 2);

    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].item.module_id, "repo:module:first");
    assert_eq!(ranked[1].item.module_id, "repo:module:second");
}

#[test]
fn ranked_symbol_matches_empty_query_keeps_input_order() {
    let symbols = vec![
        symbol_record("repo:symbol:first", "first", "Sample.first"),
        symbol_record("repo:symbol:second", "second", "Sample.second"),
    ];

    let ranked = ranked_symbol_matches("", &symbols, 2);

    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].item.symbol_id, "repo:symbol:first");
    assert_eq!(ranked[1].item.symbol_id, "repo:symbol:second");
}

#[test]
fn ranked_example_matches_empty_query_keeps_input_order() {
    let examples = vec![
        example_record("repo:example:first", "first"),
        example_record("repo:example:second", "second"),
    ];
    let metadata_lookup = BTreeMap::new();

    let ranked = ranked_example_matches("", &examples, &metadata_lookup, 2);

    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].item.example_id, "repo:example:first");
    assert_eq!(ranked[1].item.example_id, "repo:example:second");
}
