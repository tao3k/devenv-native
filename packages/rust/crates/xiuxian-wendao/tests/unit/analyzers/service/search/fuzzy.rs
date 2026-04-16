use super::super::{build_example_search, build_module_search, build_symbol_search};
use super::support::sample_search_analysis;
use crate::analyzers::{ExampleSearchQuery, ModuleSearchQuery, SymbolSearchQuery};

#[test]
fn module_search_uses_shared_tantivy_fuzzy_index_for_typos() {
    let analysis = sample_search_analysis("module-fuzzy");
    let result = build_module_search(
        &ModuleSearchQuery {
            repo_id: "module-fuzzy".to_string(),
            query: "ProjectonPkg".to_string(),
            limit: 10,
        },
        &analysis,
    );

    assert_eq!(result.modules.len(), 1);
    assert_eq!(result.modules[0].qualified_name, "ProjectionPkg");
    assert!(
        result.module_hits[0]
            .score
            .unwrap_or_else(|| panic!("shared fuzzy module search should emit a score"))
            > 0.0
    );
}

#[test]
fn symbol_search_uses_shared_tantivy_fuzzy_index_for_typos() {
    let analysis = sample_search_analysis("symbol-fuzzy");
    let result = build_symbol_search(
        &SymbolSearchQuery {
            repo_id: "symbol-fuzzy".to_string(),
            query: "slove".to_string(),
            limit: 10,
        },
        &analysis,
    );

    assert_eq!(result.symbols.len(), 1);
    assert_eq!(result.symbols[0].name, "solve");
    assert!(
        result.symbol_hits[0]
            .score
            .unwrap_or_else(|| panic!("shared fuzzy symbol search should emit a score"))
            > 0.0
    );
}

#[test]
fn example_search_uses_shared_tantivy_fuzzy_index_for_related_symbol_typos() {
    let analysis = sample_search_analysis("example-fuzzy");
    let result = build_example_search(
        &ExampleSearchQuery {
            repo_id: "example-fuzzy".to_string(),
            query: "slove".to_string(),
            limit: 10,
        },
        &analysis,
    );

    assert_eq!(result.examples.len(), 1);
    assert_eq!(result.examples[0].title, "basic");
    assert!(
        result.example_hits[0]
            .score
            .unwrap_or_else(|| panic!("shared fuzzy example search should emit a score"))
            > 0.0
    );
}
