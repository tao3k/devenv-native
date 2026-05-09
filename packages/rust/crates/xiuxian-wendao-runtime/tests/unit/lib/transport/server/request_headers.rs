use tonic::metadata::MetadataMap;

use crate::transport::{
    WENDAO_ANALYSIS_LINE_HEADER, WENDAO_ANALYSIS_PATH_HEADER, WENDAO_ANALYSIS_REPO_HEADER,
    WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER, WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER,
    WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER, WENDAO_AUTOCOMPLETE_LIMIT_HEADER,
    WENDAO_AUTOCOMPLETE_PREFIX_HEADER, WENDAO_DEFINITION_LINE_HEADER,
    WENDAO_DEFINITION_PATH_HEADER, WENDAO_DEFINITION_QUERY_HEADER,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER, WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER, WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER, WENDAO_GRAPH_DIRECTION_HEADER,
    WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER, WENDAO_GRAPH_NODE_ID_HEADER,
    WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER, WENDAO_REPO_DOC_COVERAGE_REPO_HEADER,
    WENDAO_REPO_INDEX_STATUS_REPO_HEADER, WENDAO_REPO_OVERVIEW_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
    WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
    WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER, WENDAO_REPO_SEARCH_QUERY_HEADER,
    WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_REPO_SYNC_MODE_HEADER, WENDAO_REPO_SYNC_REPO_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER, WENDAO_SEARCH_INTENT_HEADER, WENDAO_SEARCH_LIMIT_HEADER,
    WENDAO_SEARCH_QUERY_HEADER, WENDAO_SEARCH_REPO_HEADER, WENDAO_SQL_QUERY_HEADER,
    WENDAO_VFS_PATH_HEADER,
};

use super::assertions::metadata_value;

pub(super) fn build_search_metadata(query_text: &str, limit: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_search_headers(&mut metadata, query_text, limit);
    metadata
}

pub(super) fn build_repo_search_metadata(
    repo_id: &str,
    query_text: &str,
    limit: &str,
    language_filters: Option<&str>,
    path_prefixes: Option<&str>,
) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_repo_search_headers(
        &mut metadata,
        repo_id,
        query_text,
        limit,
        language_filters,
        path_prefixes,
    );
    metadata
}

pub(super) fn build_markdown_analysis_metadata(path: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_markdown_analysis_headers(&mut metadata, path);
    metadata
}

pub(super) fn build_document_extract_metadata(
    source_path: &str,
    output_dir: Option<&str>,
    force: Option<&str>,
    error_row: Option<&str>,
) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_document_extract_headers(
        &mut metadata,
        source_path,
        output_dir,
        force,
        error_row,
    );
    metadata
}

pub(super) fn build_document_extract_status_metadata(job_id: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER,
        metadata_value(job_id, "document extract job-id metadata should parse"),
    );
    metadata
}

pub(super) fn build_repo_doc_coverage_metadata(
    repo_id: &str,
    module_id: Option<&str>,
) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_repo_doc_coverage_headers(&mut metadata, repo_id, module_id);
    metadata
}

pub(super) fn build_repo_projected_retrieval_context_metadata(
    repo_id: &str,
    page_id: &str,
    node_id: Option<&str>,
    related_limit: Option<&str>,
) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_repo_projected_retrieval_context_headers(
        &mut metadata,
        repo_id,
        page_id,
        node_id,
        related_limit,
    );
    metadata
}

pub(super) fn build_repo_overview_metadata(repo_id: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_repo_overview_headers(&mut metadata, repo_id);
    metadata
}

pub(super) fn build_repo_index_status_metadata(repo_id: Option<&str>) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_repo_index_status_headers(&mut metadata, repo_id);
    metadata
}

pub(super) fn build_repo_sync_metadata(repo_id: &str, mode: Option<&str>) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_repo_sync_headers(&mut metadata, repo_id, mode);
    metadata
}

pub(super) fn build_definition_metadata(
    query_text: &str,
    source_path: Option<&str>,
    source_line: Option<&str>,
) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_definition_headers(&mut metadata, query_text, source_path, source_line);
    metadata
}

pub(super) fn build_autocomplete_metadata(prefix: &str, limit: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_autocomplete_headers(&mut metadata, prefix, limit);
    metadata
}

pub(super) fn build_sql_metadata(query_text: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_sql_headers(&mut metadata, query_text);
    metadata
}

pub(super) fn build_vfs_resolve_metadata(path: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_vfs_resolve_headers(&mut metadata, path);
    metadata
}

pub(super) fn build_vfs_content_metadata(path: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_vfs_content_headers(&mut metadata, path);
    metadata
}

pub(super) fn build_graph_neighbors_metadata(
    node_id: &str,
    direction: Option<&str>,
    hops: Option<&str>,
    limit: Option<&str>,
) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_graph_neighbors_headers(&mut metadata, node_id, direction, hops, limit);
    metadata
}

pub(super) fn build_code_ast_analysis_metadata(
    path: &str,
    repo_id: &str,
    line_hint: Option<&str>,
) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_code_ast_analysis_headers(&mut metadata, path, repo_id, line_hint);
    metadata
}

pub(super) fn build_attachment_search_metadata(
    query_text: &str,
    limit: &str,
    ext_filters: Option<&str>,
    kind_filters: Option<&str>,
    case_sensitive: Option<&str>,
) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    populate_schema_and_attachment_search_headers(
        &mut metadata,
        query_text,
        limit,
        ext_filters,
        kind_filters,
        case_sensitive,
    );
    metadata
}

pub(super) fn populate_schema_and_search_headers(
    metadata: &mut MetadataMap,
    query_text: &str,
    limit: &str,
) {
    populate_schema_and_search_headers_with_hints(metadata, query_text, limit, None, None);
}

pub(super) fn populate_schema_and_repo_search_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    query_text: &str,
    limit: &str,
    language_filters: Option<&str>,
    path_prefixes: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_REPO_SEARCH_REPO_HEADER,
        metadata_value(repo_id, "repo-search repo metadata should parse"),
    );
    metadata.insert(
        WENDAO_REPO_SEARCH_QUERY_HEADER,
        metadata_value(query_text, "repo-search query metadata should parse"),
    );
    metadata.insert(
        WENDAO_REPO_SEARCH_LIMIT_HEADER,
        metadata_value(limit, "repo-search limit metadata should parse"),
    );
    if let Some(language_filters) = language_filters {
        metadata.insert(
            WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER,
            metadata_value(
                language_filters,
                "repo-search language filters metadata should parse",
            ),
        );
    }
    if let Some(path_prefixes) = path_prefixes {
        metadata.insert(
            WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER,
            metadata_value(
                path_prefixes,
                "repo-search path prefixes metadata should parse",
            ),
        );
    }
}

pub(super) fn populate_schema_and_search_headers_with_hints(
    metadata: &mut MetadataMap,
    query_text: &str,
    limit: &str,
    intent: Option<&str>,
    repo_hint: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_SEARCH_QUERY_HEADER,
        metadata_value(query_text, "search-family query text metadata should parse"),
    );
    metadata.insert(
        WENDAO_SEARCH_LIMIT_HEADER,
        metadata_value(limit, "search-family limit metadata should parse"),
    );
    if let Some(intent) = intent {
        metadata.insert(
            WENDAO_SEARCH_INTENT_HEADER,
            metadata_value(intent, "search-family intent metadata should parse"),
        );
    }
    if let Some(repo_hint) = repo_hint {
        metadata.insert(
            WENDAO_SEARCH_REPO_HEADER,
            metadata_value(repo_hint, "search-family repo metadata should parse"),
        );
    }
}

pub(super) fn populate_schema_and_repo_doc_coverage_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    module_id: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_REPO_DOC_COVERAGE_REPO_HEADER,
        metadata_value(repo_id, "repo doc coverage repo metadata should parse"),
    );
    if let Some(module_id) = module_id {
        metadata.insert(
            WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
            metadata_value(module_id, "repo doc coverage module metadata should parse"),
        );
    }
}

pub(super) fn populate_schema_and_repo_projected_retrieval_context_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
    node_id: Option<&str>,
    related_limit: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
        metadata_value(
            repo_id,
            "repo projected retrieval-context repo metadata should parse",
        ),
    );
    metadata.insert(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
        metadata_value(
            page_id,
            "repo projected retrieval-context page id metadata should parse",
        ),
    );
    if let Some(node_id) = node_id {
        metadata.insert(
            WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
            metadata_value(
                node_id,
                "repo projected retrieval-context node id metadata should parse",
            ),
        );
    }
    if let Some(related_limit) = related_limit {
        metadata.insert(
            WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
            metadata_value(
                related_limit,
                "repo projected retrieval-context related-limit metadata should parse",
            ),
        );
    }
}

pub(super) fn populate_schema_and_repo_overview_headers(metadata: &mut MetadataMap, repo_id: &str) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_REPO_OVERVIEW_REPO_HEADER,
        metadata_value(repo_id, "repo overview repo metadata should parse"),
    );
}

pub(super) fn populate_schema_and_repo_index_status_headers(
    metadata: &mut MetadataMap,
    repo_id: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    if let Some(repo_id) = repo_id {
        metadata.insert(
            WENDAO_REPO_INDEX_STATUS_REPO_HEADER,
            metadata_value(repo_id, "repo index status repo metadata should parse"),
        );
    }
}

pub(super) fn populate_schema_and_repo_sync_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    mode: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_REPO_SYNC_REPO_HEADER,
        metadata_value(repo_id, "repo sync repo metadata should parse"),
    );
    if let Some(mode) = mode {
        metadata.insert(
            WENDAO_REPO_SYNC_MODE_HEADER,
            metadata_value(mode, "repo sync mode metadata should parse"),
        );
    }
}

pub(super) fn populate_schema_and_attachment_search_headers(
    metadata: &mut MetadataMap,
    query_text: &str,
    limit: &str,
    ext_filters: Option<&str>,
    kind_filters: Option<&str>,
    case_sensitive: Option<&str>,
) {
    populate_schema_and_search_headers(metadata, query_text, limit);
    if let Some(ext_filters) = ext_filters {
        metadata.insert(
            WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER,
            metadata_value(
                ext_filters,
                "attachment-search ext filters metadata should parse",
            ),
        );
    }
    if let Some(kind_filters) = kind_filters {
        metadata.insert(
            WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER,
            metadata_value(
                kind_filters,
                "attachment-search kind filters metadata should parse",
            ),
        );
    }
    if let Some(case_sensitive) = case_sensitive {
        metadata.insert(
            WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER,
            metadata_value(
                case_sensitive,
                "attachment-search case_sensitive metadata should parse",
            ),
        );
    }
}

pub(super) fn populate_schema_and_markdown_analysis_headers(
    metadata: &mut MetadataMap,
    path: &str,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_ANALYSIS_PATH_HEADER,
        metadata_value(path, "analysis path metadata should parse"),
    );
}

pub(super) fn populate_schema_and_document_extract_headers(
    metadata: &mut MetadataMap,
    source_path: &str,
    output_dir: Option<&str>,
    force: Option<&str>,
    error_row: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
        metadata_value(
            source_path,
            "document extract source path metadata should parse",
        ),
    );
    if let Some(output_dir) = output_dir {
        metadata.insert(
            WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
            metadata_value(
                output_dir,
                "document extract output dir metadata should parse",
            ),
        );
    }
    if let Some(force) = force {
        metadata.insert(
            WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
            metadata_value(force, "document extract force metadata should parse"),
        );
    }
    if let Some(error_row) = error_row {
        metadata.insert(
            WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
            metadata_value(
                error_row,
                "document extract error-row metadata should parse",
            ),
        );
    }
}

pub(super) fn populate_schema_and_definition_headers(
    metadata: &mut MetadataMap,
    query_text: &str,
    source_path: Option<&str>,
    source_line: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_DEFINITION_QUERY_HEADER,
        metadata_value(query_text, "definition query metadata should parse"),
    );
    if let Some(source_path) = source_path {
        metadata.insert(
            WENDAO_DEFINITION_PATH_HEADER,
            metadata_value(source_path, "definition path metadata should parse"),
        );
    }
    if let Some(source_line) = source_line {
        metadata.insert(
            WENDAO_DEFINITION_LINE_HEADER,
            metadata_value(source_line, "definition line metadata should parse"),
        );
    }
}

pub(super) fn populate_schema_and_autocomplete_headers(
    metadata: &mut MetadataMap,
    prefix: &str,
    limit: &str,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_AUTOCOMPLETE_PREFIX_HEADER,
        metadata_value(prefix, "autocomplete prefix metadata should parse"),
    );
    metadata.insert(
        WENDAO_AUTOCOMPLETE_LIMIT_HEADER,
        metadata_value(limit, "autocomplete limit metadata should parse"),
    );
}

pub(super) fn populate_schema_and_sql_headers(metadata: &mut MetadataMap, query_text: &str) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_SQL_QUERY_HEADER,
        metadata_value(query_text, "SQL query metadata should parse"),
    );
}

pub(super) fn populate_schema_and_vfs_resolve_headers(metadata: &mut MetadataMap, path: &str) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_VFS_PATH_HEADER,
        metadata_value(path, "VFS resolve path metadata should parse"),
    );
}

pub(super) fn populate_schema_and_vfs_content_headers(metadata: &mut MetadataMap, path: &str) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_VFS_PATH_HEADER,
        metadata_value(path, "VFS content path metadata should parse"),
    );
}

pub(super) fn populate_schema_and_graph_neighbors_headers(
    metadata: &mut MetadataMap,
    node_id: &str,
    direction: Option<&str>,
    hops: Option<&str>,
    limit: Option<&str>,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata should parse"),
    );
    metadata.insert(
        WENDAO_GRAPH_NODE_ID_HEADER,
        metadata_value(node_id, "graph-neighbors node id metadata should parse"),
    );
    if let Some(direction) = direction {
        metadata.insert(
            WENDAO_GRAPH_DIRECTION_HEADER,
            metadata_value(direction, "graph-neighbors direction metadata should parse"),
        );
    }
    if let Some(hops) = hops {
        metadata.insert(
            WENDAO_GRAPH_HOPS_HEADER,
            metadata_value(hops, "graph-neighbors hops metadata should parse"),
        );
    }
    if let Some(limit) = limit {
        metadata.insert(
            WENDAO_GRAPH_LIMIT_HEADER,
            metadata_value(limit, "graph-neighbors limit metadata should parse"),
        );
    }
}

pub(super) fn populate_schema_and_code_ast_analysis_headers(
    metadata: &mut MetadataMap,
    path: &str,
    repo_id: &str,
    line_hint: Option<&str>,
) {
    populate_schema_and_markdown_analysis_headers(metadata, path);
    metadata.insert(
        WENDAO_ANALYSIS_REPO_HEADER,
        metadata_value(repo_id, "analysis repo metadata should parse"),
    );
    if let Some(line_hint) = line_hint {
        metadata.insert(
            WENDAO_ANALYSIS_LINE_HEADER,
            metadata_value(line_hint, "analysis line metadata should parse"),
        );
    }
}
