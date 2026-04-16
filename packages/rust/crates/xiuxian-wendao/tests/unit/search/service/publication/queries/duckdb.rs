use std::iter::FromIterator;

use serial_test::serial;

use super::support::{publish_repo_content_chunks, publish_repo_entities, repo_search_service, *};

#[tokio::test]
#[serial]
async fn search_repo_entities_reads_hits_from_published_table_with_duckdb_repo_query_engine() {
    let _temp = super::support::write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/repo-entity-query-tmp"
threads = 2
"#,
    )
    .unwrap_or_else(|error| panic!("write duckdb runtime override: {error}"));

    let service = repo_search_service();
    publish_repo_entities(&service).await;

    let kind_filters = HashSet::from_iter([String::from("function")]);
    let hits = ok_or_panic(
        service
            .search_repo_entities("alpha/repo", "reexport", &HashSet::new(), &kind_filters, 5)
            .await,
        "query repo entities with duckdb repo query engine",
    );

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].doc_type.as_deref(), Some("symbol"));
    assert_eq!(hits[0].stem, "reexport");
    assert_eq!(hits[0].path, "src/BaseModelica.jl");
}

#[tokio::test]
#[serial]
async fn search_repo_content_chunks_with_filters_applies_sql_native_repo_filters_with_duckdb_repo_query_engine()
 {
    let _temp = super::support::write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/repo-content-query-tmp"
threads = 2
"#,
    )
    .unwrap_or_else(|error| panic!("write duckdb runtime override: {error}"));

    let service = repo_search_service();
    publish_repo_content_chunks(&service).await;

    let hits = ok_or_panic(
        service
            .search_repo_content_chunks_with_filters(
                "alpha/repo",
                "reexport",
                &HashSet::new(),
                &RepoContentChunkSearchFilters {
                    path_prefixes: HashSet::from(["src/".to_string()]),
                    filename_filters: HashSet::from(["BaseModelica.jl".to_string()]),
                    ..RepoContentChunkSearchFilters::default()
                },
                5,
            )
            .await,
        "query repo content with duckdb repo query engine",
    );

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "src/BaseModelica.jl");
    assert_eq!(hits[0].title.as_deref(), Some("src/BaseModelica.jl"));
}
