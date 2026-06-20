use super::{
    ANALYSIS_MARKDOWN_ROUTE, ANALYSIS_REPO_INDEX_ROUTE,
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    ANALYSIS_REPO_SYNC_ROUTE, GRAPH_NEIGHBORS_DEFAULT_HOPS, GRAPH_NEIGHBORS_DEFAULT_LIMIT,
    GRAPH_NEIGHBORS_ROUTE, ONTOLOGY_DATASET_MATERIALIZE_ROUTE, QUERY_SQL_ROUTE,
    REPO_PROJECTED_RETRIEVAL_CONTEXT_DEFAULT_RELATED_LIMIT, REPO_SEARCH_DEFAULT_LIMIT,
    REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_LANGUAGE_COLUMN, REPO_SEARCH_PATH_COLUMN,
    REPO_SEARCH_ROUTE, REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TITLE_COLUMN, RERANK_ROUTE,
    SEARCH_ATTACHMENTS_ROUTE, SEARCH_AUTOCOMPLETE_ROUTE, SEARCH_DEFINITION_ROUTE,
    SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE, SEARCH_REFERENCES_ROUTE, SEARCH_SYMBOLS_ROUTE,
    VFS_CONTENT_ROUTE, VFS_RESOLVE_ROUTE, WENDAO_ANALYSIS_LINE_HEADER, WENDAO_ANALYSIS_PATH_HEADER,
    WENDAO_ANALYSIS_REPO_HEADER, WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER,
    WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER, WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER,
    WENDAO_AUTOCOMPLETE_LIMIT_HEADER, WENDAO_AUTOCOMPLETE_PREFIX_HEADER,
    WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER, WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER,
    WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER, WENDAO_DATASET_ONTOLOGY_PAYLOAD_ID_HEADER,
    WENDAO_DEFINITION_LINE_HEADER, WENDAO_DEFINITION_PATH_HEADER, WENDAO_DEFINITION_QUERY_HEADER,
    WENDAO_GRAPH_DIRECTION_HEADER, WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER,
    WENDAO_GRAPH_NODE_ID_HEADER, WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
    WENDAO_REPO_DOC_COVERAGE_REPO_HEADER, WENDAO_REPO_INDEX_REFRESH_HEADER,
    WENDAO_REPO_INDEX_REPO_HEADER, WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
    WENDAO_REPO_SEARCH_FILENAME_FILTERS_HEADER, WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER,
    WENDAO_REPO_SEARCH_LIMIT_HEADER, WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER,
    WENDAO_REPO_SEARCH_QUERY_HEADER, WENDAO_REPO_SEARCH_REPO_HEADER,
    WENDAO_REPO_SEARCH_TAG_FILTERS_HEADER, WENDAO_REPO_SEARCH_TITLE_FILTERS_HEADER,
    WENDAO_REPO_SYNC_MODE_HEADER, WENDAO_REPO_SYNC_REPO_HEADER, WENDAO_RERANK_DIMENSION_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER, WENDAO_SEARCH_LIMIT_HEADER, WENDAO_SEARCH_QUERY_HEADER,
    WENDAO_SQL_QUERY_HEADER, WENDAO_VFS_PATH_HEADER, flight_descriptor_path,
    normalize_flight_route,
};

#[cfg(feature = "transport")]
use crate::transport::query_contract::{
    RERANK_REQUEST_DOC_ID_COLUMN, RERANK_REQUEST_EMBEDDING_COLUMN,
    RERANK_REQUEST_QUERY_EMBEDDING_COLUMN, RERANK_REQUEST_VECTOR_SCORE_COLUMN,
    RERANK_RESPONSE_DOC_ID_COLUMN, RERANK_RESPONSE_FINAL_SCORE_COLUMN, RERANK_RESPONSE_RANK_COLUMN,
    RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN, RERANK_RESPONSE_VECTOR_SCORE_COLUMN,
};

#[test]
fn query_contract_exposes_stable_headers() {
    query_contract_exposes_core_headers();
    query_contract_exposes_graph_headers();
    query_contract_exposes_repo_headers();
}

fn query_contract_exposes_core_headers() {
    assert_eq!(WENDAO_SCHEMA_VERSION_HEADER, "x-wendao-schema-version");
    assert_eq!(WENDAO_SEARCH_QUERY_HEADER, "x-wendao-search-query");
    assert_eq!(WENDAO_SEARCH_LIMIT_HEADER, "x-wendao-search-limit");
    assert_eq!(WENDAO_SQL_QUERY_HEADER, "x-wendao-sql-query");
    assert_eq!(WENDAO_DEFINITION_QUERY_HEADER, "x-wendao-definition-query");
    assert_eq!(WENDAO_DEFINITION_PATH_HEADER, "x-wendao-definition-path");
    assert_eq!(WENDAO_DEFINITION_LINE_HEADER, "x-wendao-definition-line");
    assert_eq!(
        WENDAO_AUTOCOMPLETE_PREFIX_HEADER,
        "x-wendao-autocomplete-prefix"
    );
    assert_eq!(
        WENDAO_AUTOCOMPLETE_LIMIT_HEADER,
        "x-wendao-autocomplete-limit"
    );
    assert_eq!(WENDAO_VFS_PATH_HEADER, "x-wendao-vfs-path");
    assert_eq!(
        WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER,
        "x-wendao-dataset-ontology-contract-id"
    );
    assert_eq!(
        WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER,
        "x-wendao-dataset-ontology-mapping-id"
    );
    assert_eq!(
        WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER,
        "x-wendao-dataset-ontology-manifest"
    );
    assert_eq!(
        WENDAO_DATASET_ONTOLOGY_PAYLOAD_ID_HEADER,
        "x-wendao-dataset-ontology-payload-id"
    );
    assert_eq!(WENDAO_ANALYSIS_PATH_HEADER, "x-wendao-analysis-path");
    assert_eq!(WENDAO_ANALYSIS_REPO_HEADER, "x-wendao-analysis-repo");
    assert_eq!(WENDAO_ANALYSIS_LINE_HEADER, "x-wendao-analysis-line");
    assert_eq!(
        WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER,
        "x-wendao-attachment-search-ext-filters"
    );
    assert_eq!(
        WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER,
        "x-wendao-attachment-search-kind-filters"
    );
    assert_eq!(
        WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER,
        "x-wendao-attachment-search-case-sensitive"
    );
    assert_eq!(
        WENDAO_RERANK_DIMENSION_HEADER,
        "x-wendao-rerank-embedding-dimension"
    );
}

fn query_contract_exposes_graph_headers() {
    assert_eq!(WENDAO_GRAPH_NODE_ID_HEADER, "x-wendao-graph-node-id");
    assert_eq!(WENDAO_GRAPH_DIRECTION_HEADER, "x-wendao-graph-direction");
    assert_eq!(WENDAO_GRAPH_HOPS_HEADER, "x-wendao-graph-hops");
    assert_eq!(WENDAO_GRAPH_LIMIT_HEADER, "x-wendao-graph-limit");
}

fn query_contract_exposes_repo_headers() {
    assert_eq!(
        WENDAO_REPO_DOC_COVERAGE_REPO_HEADER,
        "x-wendao-repo-doc-coverage-repo"
    );
    assert_eq!(
        WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
        "x-wendao-repo-doc-coverage-module"
    );
    assert_eq!(WENDAO_REPO_INDEX_REPO_HEADER, "x-wendao-repo-index-repo");
    assert_eq!(
        WENDAO_REPO_INDEX_REFRESH_HEADER,
        "x-wendao-repo-index-refresh"
    );
    assert_eq!(
        WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
        "x-wendao-repo-index-request-id"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
        "x-wendao-repo-projected-page-index-tree-repo"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
        "x-wendao-repo-projected-page-index-tree-page-id"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
        "x-wendao-repo-projected-retrieval-context-repo"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
        "x-wendao-repo-projected-retrieval-context-page-id"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
        "x-wendao-repo-projected-retrieval-context-node-id"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
        "x-wendao-repo-projected-retrieval-context-related-limit"
    );
    assert_eq!(
        WENDAO_REPO_SEARCH_QUERY_HEADER,
        "x-wendao-repo-search-query"
    );
    assert_eq!(
        WENDAO_REPO_SEARCH_LIMIT_HEADER,
        "x-wendao-repo-search-limit"
    );
    assert_eq!(WENDAO_REPO_SEARCH_REPO_HEADER, "x-wendao-repo-search-repo");
    assert_eq!(
        WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER,
        "x-wendao-repo-search-language-filters"
    );
    assert_eq!(
        WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER,
        "x-wendao-repo-search-path-prefixes"
    );
    assert_eq!(
        WENDAO_REPO_SEARCH_TITLE_FILTERS_HEADER,
        "x-wendao-repo-search-title-filters"
    );
    assert_eq!(
        WENDAO_REPO_SEARCH_TAG_FILTERS_HEADER,
        "x-wendao-repo-search-tag-filters"
    );
    assert_eq!(
        WENDAO_REPO_SEARCH_FILENAME_FILTERS_HEADER,
        "x-wendao-repo-search-filename-filters"
    );
    assert_eq!(WENDAO_REPO_SYNC_REPO_HEADER, "x-wendao-repo-sync-repo");
    assert_eq!(WENDAO_REPO_SYNC_MODE_HEADER, "x-wendao-repo-sync-mode");
}

#[test]
fn query_contract_exposes_stable_routes() {
    assert_eq!(REPO_SEARCH_ROUTE, "/search/repos/main");
    assert_eq!(SEARCH_INTENT_ROUTE, "/search/intent");
    assert_eq!(SEARCH_KNOWLEDGE_ROUTE, "/search/knowledge");
    assert_eq!(SEARCH_ATTACHMENTS_ROUTE, "/search/attachments");
    assert_eq!(SEARCH_REFERENCES_ROUTE, "/search/references");
    assert_eq!(SEARCH_SYMBOLS_ROUTE, "/search/symbols");
    assert_eq!(SEARCH_DEFINITION_ROUTE, "/search/definition");
    assert_eq!(SEARCH_AUTOCOMPLETE_ROUTE, "/search/autocomplete");
    assert_eq!(QUERY_SQL_ROUTE, "/query/sql");
    assert_eq!(
        ONTOLOGY_DATASET_MATERIALIZE_ROUTE,
        "/ontology/dataset/materialize"
    );
    assert_eq!(VFS_RESOLVE_ROUTE, "/vfs/resolve");
    assert_eq!(VFS_CONTENT_ROUTE, "/vfs/content");
    assert_eq!(GRAPH_NEIGHBORS_ROUTE, "/graph/neighbors");
    assert_eq!(ANALYSIS_MARKDOWN_ROUTE, "/analysis/markdown");
    assert_eq!(ANALYSIS_REPO_INDEX_ROUTE, "/analysis/repo-index");
    assert_eq!(ANALYSIS_REPO_SYNC_ROUTE, "/analysis/repo-sync");
    assert_eq!(
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        "/analysis/repo-projected-page-index-tree"
    );
    assert_eq!(
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
        "/analysis/repo-projected-retrieval-context"
    );
    assert_eq!(RERANK_ROUTE, "/rerank");
}

#[test]
fn query_contract_exposes_stable_defaults_and_columns() {
    assert_eq!(REPO_SEARCH_DEFAULT_LIMIT, 10);
    assert_eq!(REPO_PROJECTED_RETRIEVAL_CONTEXT_DEFAULT_RELATED_LIMIT, 5);
    assert_eq!(GRAPH_NEIGHBORS_DEFAULT_HOPS, 2);
    assert_eq!(GRAPH_NEIGHBORS_DEFAULT_LIMIT, 50);
    assert_eq!(REPO_SEARCH_DOC_ID_COLUMN, "doc_id");
    assert_eq!(REPO_SEARCH_PATH_COLUMN, "path");
    assert_eq!(REPO_SEARCH_TITLE_COLUMN, "title");
    assert_eq!(REPO_SEARCH_SCORE_COLUMN, "score");
    assert_eq!(REPO_SEARCH_LANGUAGE_COLUMN, "language");
    #[cfg(feature = "transport")]
    {
        assert_eq!(RERANK_REQUEST_DOC_ID_COLUMN, "doc_id");
        assert_eq!(RERANK_REQUEST_VECTOR_SCORE_COLUMN, "vector_score");
        assert_eq!(RERANK_REQUEST_EMBEDDING_COLUMN, "embedding");
        assert_eq!(RERANK_REQUEST_QUERY_EMBEDDING_COLUMN, "query_embedding");
        assert_eq!(RERANK_RESPONSE_DOC_ID_COLUMN, "doc_id");
        assert_eq!(RERANK_RESPONSE_VECTOR_SCORE_COLUMN, "vector_score");
        assert_eq!(RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN, "semantic_score");
        assert_eq!(RERANK_RESPONSE_FINAL_SCORE_COLUMN, "final_score");
        assert_eq!(RERANK_RESPONSE_RANK_COLUMN, "rank");
    }
}

#[test]
fn normalize_flight_route_enforces_canonical_leading_slash() {
    assert_eq!(
        normalize_flight_route("search/repos/main").as_deref(),
        Ok("/search/repos/main")
    );
    assert_eq!(normalize_flight_route("/rerank").as_deref(), Ok("/rerank"));
}

#[test]
fn normalize_flight_route_rejects_empty_segments() {
    assert!(normalize_flight_route("").is_err());
    assert!(normalize_flight_route("/").is_err());
}

#[test]
fn descriptor_path_matches_stable_query_route() {
    assert_eq!(
        flight_descriptor_path(REPO_SEARCH_ROUTE),
        Ok(vec![
            "search".to_string(),
            "repos".to_string(),
            "main".to_string()
        ])
    );
    assert_eq!(
        flight_descriptor_path(SEARCH_INTENT_ROUTE),
        Ok(vec!["search".to_string(), "intent".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(SEARCH_KNOWLEDGE_ROUTE),
        Ok(vec!["search".to_string(), "knowledge".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(SEARCH_ATTACHMENTS_ROUTE),
        Ok(vec!["search".to_string(), "attachments".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(SEARCH_REFERENCES_ROUTE),
        Ok(vec!["search".to_string(), "references".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(SEARCH_SYMBOLS_ROUTE),
        Ok(vec!["search".to_string(), "symbols".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(SEARCH_DEFINITION_ROUTE),
        Ok(vec!["search".to_string(), "definition".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(SEARCH_AUTOCOMPLETE_ROUTE),
        Ok(vec!["search".to_string(), "autocomplete".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(QUERY_SQL_ROUTE),
        Ok(vec!["query".to_string(), "sql".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(VFS_RESOLVE_ROUTE),
        Ok(vec!["vfs".to_string(), "resolve".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(VFS_CONTENT_ROUTE),
        Ok(vec!["vfs".to_string(), "content".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(GRAPH_NEIGHBORS_ROUTE),
        Ok(vec!["graph".to_string(), "neighbors".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(ANALYSIS_MARKDOWN_ROUTE),
        Ok(vec!["analysis".to_string(), "markdown".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(ANALYSIS_REPO_INDEX_ROUTE),
        Ok(vec!["analysis".to_string(), "repo-index".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(ANALYSIS_REPO_SYNC_ROUTE),
        Ok(vec!["analysis".to_string(), "repo-sync".to_string()])
    );
    assert_eq!(
        flight_descriptor_path(ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE),
        Ok(vec![
            "analysis".to_string(),
            "repo-projected-page-index-tree".to_string()
        ])
    );
    assert_eq!(
        flight_descriptor_path(ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE),
        Ok(vec![
            "analysis".to_string(),
            "repo-projected-retrieval-context".to_string()
        ])
    );
    assert_eq!(
        flight_descriptor_path(RERANK_ROUTE),
        Ok(vec!["rerank".to_string()])
    );
}
