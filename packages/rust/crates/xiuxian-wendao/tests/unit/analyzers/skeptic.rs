use crate::analyzers::RepoSymbolKind;
use crate::analyzers::audit_symbols;
use crate::analyzers::{DocRecord, RelationKind, RelationRecord, SymbolRecord};
use std::collections::BTreeMap;

#[test]
fn test_audit_symbols_verified() {
    let symbols = vec![SymbolRecord {
        repo_id: "test".to_string(),
        symbol_id: "sym1".to_string(),
        module_id: None,
        name: "solve_ode".to_string(),
        qualified_name: "solve_ode".to_string(),
        kind: RepoSymbolKind::Function,
        path: "src/main.jl".to_string(),
        line_start: None,
        line_end: None,
        signature: None,
        audit_status: None,
        verification_state: None,
        attributes: BTreeMap::new(),
    }];

    let docs = vec![DocRecord {
        repo_id: "test".to_string(),
        doc_id: "doc1".to_string(),
        title: "How to use solve_ode".to_string(),
        path: "docs/solve.md".to_string(),
        format: None,
        doc_target: None,
    }];

    let relations = vec![RelationRecord {
        repo_id: "test".to_string(),
        source_id: "doc1".to_string(),
        target_id: "sym1".to_string(),
        kind: RelationKind::Documents,
    }];

    let results = audit_symbols(&symbols, &docs, &relations);
    assert_eq!(
        results
            .get("sym1")
            .unwrap_or_else(|| panic!("sym1 audit result should be present")),
        "verified"
    );
}

#[test]
fn test_audit_symbols_unverified() {
    let symbols = vec![SymbolRecord {
        repo_id: "test".to_string(),
        symbol_id: "sym1".to_string(),
        module_id: None,
        name: "solve_ode".to_string(),
        qualified_name: "solve_ode".to_string(),
        kind: RepoSymbolKind::Function,
        path: "src/main.jl".to_string(),
        line_start: None,
        line_end: None,
        signature: None,
        audit_status: None,
        verification_state: None,
        attributes: BTreeMap::new(),
    }];

    let docs = vec![DocRecord {
        repo_id: "test".to_string(),
        doc_id: "doc1".to_string(),
        title: "General Tutorial".to_string(),
        path: "docs/tutorial.md".to_string(),
        format: None,
        doc_target: None,
    }];

    let relations = vec![RelationRecord {
        repo_id: "test".to_string(),
        source_id: "doc1".to_string(),
        target_id: "sym1".to_string(),
        kind: RelationKind::Documents,
    }];

    let results = audit_symbols(&symbols, &docs, &relations);
    assert_eq!(
        results
            .get("sym1")
            .unwrap_or_else(|| panic!("sym1 audit result should be present")),
        "unverified"
    );
}
