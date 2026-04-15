use super::*;

#[test]
fn test_unified_symbol_creation() {
    let proj = UnifiedSymbol::new_project("my_func", "fn", "src/lib.rs:42", "mycrate");
    assert!(proj.is_project());
    assert_eq!(proj.crate_name, "mycrate");

    let ext = UnifiedSymbol::new_external("spawn", "fn", "task_join_set.rs:1", "tokio");
    assert!(ext.is_external());
}

#[test]
fn test_unified_search() {
    let mut index = UnifiedSymbolIndex::new();

    index.add_project_symbol("my_cool_function", "fn", "src/lib.rs:10", "mycrate");
    index.add_external_symbol("spawn_local", "fn", "lib.rs:1", "tokio");
    index.add_external_symbol("spawn_blocking", "fn", "lib.rs:5", "tokio");

    // 1. Exact match
    let results = index.search_unified("my_cool_function", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "my_cool_function");

    // 2. Tokenized match (Full-Text Search)
    // 'cool' should match 'my_cool_function' because we use TEXT schema with default tokenizer
    let cool_results = index.search_unified("cool", 10);
    assert!(
        !cool_results.is_empty(),
        "FTS should find 'my_cool_function' for query 'cool'"
    );
    assert_eq!(cool_results[0].name, "my_cool_function");

    // 3. Filtering by source
    let proj_results = index.search_project("spawn", 10);
    assert_eq!(proj_results.len(), 0);

    let ext_results = index.search_external("spawn", 10);
    assert_eq!(ext_results.len(), 2);

    // 4. Tokenization on external symbols
    let local_results = index.search_unified("local", 10);
    assert!(local_results.iter().any(|s| s.name == "spawn_local"));
}

#[test]
fn test_external_usage() {
    let mut index = UnifiedSymbolIndex::new();

    index.record_external_usage("tokio", "spawn", "src/main.rs:10");
    index.record_external_usage("tokio", "spawn", "src/worker.rs:5");

    let usage = index.find_external_usage("tokio");
    assert_eq!(usage.len(), 2);
    assert!(usage.iter().any(|s| s == "src/main.rs:10"));
    assert!(usage.iter().any(|s| s == "src/worker.rs:5"));
}

#[test]
fn test_unified_search_fuzzy_options() {
    let mut index = UnifiedSymbolIndex::new();
    index.add_external_symbol("spawn_local", "fn", "lib.rs:1", "tokio");

    let results = index.search_unified_with_options(
        "spwan_local",
        10,
        crate::FuzzySearchOptions::symbol_search(),
    );
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "spawn_local");
}

#[test]
fn test_unified_search_merges_tantivy_and_memory_hits_without_duplicates() {
    let mut index = UnifiedSymbolIndex::new();
    index.add_project_symbol("AlphaService", "struct", "src/lib.rs:1", "demo");
    index.add_project_symbol("alpha_handler", "fn", "src/lib.rs:2", "demo");
    index.add_project_symbol("AlphaClient", "struct", "src/tool.py:1", "demo");
    index.add_project_symbol("alpha_helper", "fn", "src/tool.py:4", "demo");

    let results = index.search_unified("alpha", 10);
    let mut names = results
        .iter()
        .map(|symbol| symbol.name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();

    assert_eq!(results.len(), 4);
    assert_eq!(
        names,
        vec![
            "AlphaClient",
            "AlphaService",
            "alpha_handler",
            "alpha_helper"
        ]
    );
}

#[test]
fn test_add_symbols_batch_indexes_memory_and_search_documents() {
    let mut index = UnifiedSymbolIndex::new();
    index.add_symbols_batch(vec![
        UnifiedSymbol::new_project("AlphaService", "struct", "src/lib.rs:1", "demo"),
        UnifiedSymbol::new_project("alpha_handler", "fn", "src/lib.rs:2", "demo"),
        UnifiedSymbol::new_external("spawn_local", "fn", "lib.rs:1", "tokio"),
    ]);

    let unified_results = index.search_unified("alpha", 10);
    assert_eq!(unified_results.len(), 2);
    assert!(
        unified_results
            .iter()
            .any(|symbol| symbol.name == "AlphaService")
    );
    assert!(
        unified_results
            .iter()
            .any(|symbol| symbol.name == "alpha_handler")
    );

    let external_results = index.search_external("spawn", 10);
    assert_eq!(external_results.len(), 1);
    assert_eq!(external_results[0].name, "spawn_local");

    let stats = index.stats();
    assert_eq!(stats.total_symbols, 3);
    assert_eq!(stats.project_symbols, 2);
    assert_eq!(stats.external_symbols, 1);
}

#[test]
fn test_stats() {
    let mut index = UnifiedSymbolIndex::new();
    index.add_project_symbol("f1", "fn", "loc", "c1");
    index.add_project_symbol("f2", "fn", "loc", "c1");
    index.add_external_symbol("e1", "fn", "loc", "c2");

    index.record_external_usage("c2", "e1", "src/main.rs:10");

    let stats = index.stats();
    assert_eq!(stats.total_symbols, 3);
    assert_eq!(stats.project_symbols, 2);
    assert_eq!(stats.external_symbols, 1);
    assert_eq!(stats.external_crates, 1);
}
