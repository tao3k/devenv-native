use super::support::{ok_or_panic, sample_cache_key, sample_search_analysis};
use crate::analyzers::service::search::{
    build_example_search, build_example_search_with_artifacts, build_import_search,
    build_import_search_with_artifacts, build_module_search, build_module_search_with_artifacts,
    build_symbol_search, build_symbol_search_with_artifacts, repository_search_artifacts,
};
use crate::analyzers::{
    ExampleSearchQuery, ImportSearchQuery, ModuleSearchQuery, SymbolSearchQuery,
};

#[test]
fn module_search_with_artifacts_matches_direct_search() {
    let analysis = sample_search_analysis("module-artifacts");
    let query = ModuleSearchQuery {
        repo_id: "module-artifacts".to_string(),
        query: "ProjectonPkg".to_string(),
        limit: 10,
    };
    let artifacts = ok_or_panic(
        repository_search_artifacts(&sample_cache_key("module-artifacts"), &analysis),
        "artifacts should build",
    );

    assert_eq!(
        build_module_search(&query, &analysis),
        build_module_search_with_artifacts(&query, &analysis, artifacts.as_ref())
    );
}

#[test]
fn symbol_search_with_artifacts_matches_direct_search() {
    let analysis = sample_search_analysis("symbol-artifacts");
    let query = SymbolSearchQuery {
        repo_id: "symbol-artifacts".to_string(),
        query: "slove".to_string(),
        limit: 10,
    };
    let artifacts = ok_or_panic(
        repository_search_artifacts(&sample_cache_key("symbol-artifacts"), &analysis),
        "artifacts should build",
    );

    assert_eq!(
        build_symbol_search(&query, &analysis),
        build_symbol_search_with_artifacts(&query, &analysis, artifacts.as_ref())
    );
}

#[test]
fn example_search_with_artifacts_matches_direct_search() {
    let analysis = sample_search_analysis("example-artifacts");
    let query = ExampleSearchQuery {
        repo_id: "example-artifacts".to_string(),
        query: "slove".to_string(),
        limit: 10,
    };
    let artifacts = ok_or_panic(
        repository_search_artifacts(&sample_cache_key("example-artifacts"), &analysis),
        "artifacts should build",
    );

    assert_eq!(
        build_example_search(&query, &analysis),
        build_example_search_with_artifacts(&query, &analysis, artifacts.as_ref())
    );
}

#[test]
fn import_search_with_artifacts_matches_direct_search() {
    let analysis = sample_search_analysis("import-artifacts");
    let query = ImportSearchQuery {
        repo_id: "import-artifacts".to_string(),
        package: Some("SciMLBase".to_string()),
        module: Some("BaseModelica".to_string()),
        limit: 10,
    };
    let artifacts = ok_or_panic(
        repository_search_artifacts(&sample_cache_key("import-artifacts"), &analysis),
        "artifacts should build",
    );

    assert_eq!(
        build_import_search(&query, &analysis),
        build_import_search_with_artifacts(&query, &analysis, artifacts.as_ref())
    );
}
