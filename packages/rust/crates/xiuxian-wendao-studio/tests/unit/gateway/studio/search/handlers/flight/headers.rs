use crate::transport::{
    WENDAO_ANALYSIS_LINE_HEADER, WENDAO_ANALYSIS_REPO_HEADER, WENDAO_REFINE_DOC_ENTITY_ID_HEADER,
    WENDAO_REFINE_DOC_REPO_HEADER, WENDAO_REFINE_DOC_USER_HINTS_HEADER,
    WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER, WENDAO_REPO_DOC_COVERAGE_REPO_HEADER,
    WENDAO_REPO_OVERVIEW_REPO_HEADER, WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
};
use crate::transport::{
    WENDAO_ANALYSIS_PATH_HEADER, WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER,
    WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER, WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER,
    WENDAO_AUTOCOMPLETE_LIMIT_HEADER, WENDAO_AUTOCOMPLETE_PREFIX_HEADER,
    WENDAO_DEFINITION_LINE_HEADER, WENDAO_DEFINITION_PATH_HEADER, WENDAO_DEFINITION_QUERY_HEADER,
    WENDAO_GRAPH_DIRECTION_HEADER, WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER,
    WENDAO_GRAPH_NODE_ID_HEADER, WENDAO_REPO_INDEX_REFRESH_HEADER, WENDAO_REPO_INDEX_REPO_HEADER,
    WENDAO_REPO_INDEX_REQUEST_ID_HEADER, WENDAO_REPO_INDEX_STATUS_REPO_HEADER,
    WENDAO_REPO_SEARCH_LIMIT_HEADER, WENDAO_REPO_SEARCH_QUERY_HEADER,
    WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_REPO_SYNC_MODE_HEADER, WENDAO_REPO_SYNC_REPO_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER, WENDAO_SEARCH_LIMIT_HEADER, WENDAO_SEARCH_QUERY_HEADER,
    WENDAO_VFS_PATH_HEADER,
};
use tonic::metadata::MetadataMap;

fn insert_header(metadata: &mut MetadataMap, header: &'static str, value: &str, context: &str) {
    metadata.insert(
        header,
        value
            .parse()
            .unwrap_or_else(|error| panic!("{context}: {error}")),
    );
}

pub(super) fn populate_search_headers(metadata: &mut MetadataMap, query_text: &str, limit: usize) {
    insert_header(
        metadata,
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2",
        "schema metadata",
    );
    insert_header(
        metadata,
        WENDAO_SEARCH_QUERY_HEADER,
        query_text,
        "query metadata",
    );
    insert_header(
        metadata,
        WENDAO_SEARCH_LIMIT_HEADER,
        &limit.to_string(),
        "limit metadata",
    );
}

pub(super) fn populate_repo_search_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    query_text: &str,
    limit: usize,
) {
    populate_schema_headers(metadata);
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_REPO_HEADER,
        repo_id,
        "repo search repo metadata",
    );
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_QUERY_HEADER,
        query_text,
        "repo search query metadata",
    );
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_LIMIT_HEADER,
        &limit.to_string(),
        "repo search limit metadata",
    );
}

pub(super) fn populate_attachment_headers(
    metadata: &mut MetadataMap,
    query_text: &str,
    limit: usize,
) {
    populate_search_headers(metadata, query_text, limit);
    insert_header(
        metadata,
        WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER,
        "png",
        "ext metadata",
    );
    insert_header(
        metadata,
        WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER,
        "image",
        "kind metadata",
    );
    insert_header(
        metadata,
        WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER,
        "false",
        "case metadata",
    );
}

pub(super) fn populate_definition_headers(
    metadata: &mut MetadataMap,
    query_text: &str,
    source_path: &str,
    source_line: usize,
) {
    insert_header(
        metadata,
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2",
        "schema metadata",
    );
    insert_header(
        metadata,
        WENDAO_DEFINITION_QUERY_HEADER,
        query_text,
        "definition query metadata",
    );
    insert_header(
        metadata,
        WENDAO_DEFINITION_PATH_HEADER,
        source_path,
        "definition path metadata",
    );
    insert_header(
        metadata,
        WENDAO_DEFINITION_LINE_HEADER,
        &source_line.to_string(),
        "definition line metadata",
    );
}

pub(super) fn populate_autocomplete_headers(
    metadata: &mut MetadataMap,
    prefix: &str,
    limit: usize,
) {
    insert_header(
        metadata,
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2",
        "schema metadata",
    );
    insert_header(
        metadata,
        WENDAO_AUTOCOMPLETE_PREFIX_HEADER,
        prefix,
        "autocomplete prefix metadata",
    );
    insert_header(
        metadata,
        WENDAO_AUTOCOMPLETE_LIMIT_HEADER,
        &limit.to_string(),
        "autocomplete limit metadata",
    );
}

pub(super) fn populate_vfs_resolve_headers(metadata: &mut MetadataMap, path: &str) {
    insert_header(
        metadata,
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2",
        "schema metadata",
    );
    insert_header(metadata, WENDAO_VFS_PATH_HEADER, path, "VFS path metadata");
}

pub(super) fn populate_vfs_content_headers(metadata: &mut MetadataMap, path: &str) {
    populate_vfs_resolve_headers(metadata, path);
}

fn populate_schema_headers(metadata: &mut MetadataMap) {
    insert_header(
        metadata,
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2",
        "schema metadata",
    );
}

pub(super) fn populate_vfs_scan_headers(metadata: &mut MetadataMap) {
    populate_schema_headers(metadata);
}

pub(super) fn populate_graph_neighbors_headers(
    metadata: &mut MetadataMap,
    node_id: &str,
    direction: &str,
    hops: usize,
    limit: usize,
) {
    populate_schema_headers(metadata);
    insert_header(
        metadata,
        WENDAO_GRAPH_NODE_ID_HEADER,
        node_id,
        "graph node id metadata",
    );
    insert_header(
        metadata,
        WENDAO_GRAPH_DIRECTION_HEADER,
        direction,
        "graph direction metadata",
    );
    insert_header(
        metadata,
        WENDAO_GRAPH_HOPS_HEADER,
        &hops.to_string(),
        "graph hops metadata",
    );
    insert_header(
        metadata,
        WENDAO_GRAPH_LIMIT_HEADER,
        &limit.to_string(),
        "graph limit metadata",
    );
}

pub(super) fn populate_topology_3d_headers(metadata: &mut MetadataMap) {
    populate_schema_headers(metadata);
}

pub(super) fn populate_markdown_analysis_headers(metadata: &mut MetadataMap, path: &str) {
    insert_header(
        metadata,
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2",
        "schema metadata",
    );
    insert_header(
        metadata,
        WENDAO_ANALYSIS_PATH_HEADER,
        path,
        "analysis path metadata",
    );
}

pub(super) fn populate_code_ast_analysis_headers(
    metadata: &mut MetadataMap,
    path: &str,
    repo_id: &str,
    line_hint: Option<usize>,
) {
    populate_markdown_analysis_headers(metadata, path);
    insert_header(
        metadata,
        WENDAO_ANALYSIS_REPO_HEADER,
        repo_id,
        "analysis repo metadata",
    );
    if let Some(line_hint) = line_hint {
        insert_header(
            metadata,
            WENDAO_ANALYSIS_LINE_HEADER,
            &line_hint.to_string(),
            "analysis line metadata",
        );
    }
}

pub(super) fn populate_repo_doc_coverage_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    module_id: Option<&str>,
) {
    insert_header(
        metadata,
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2",
        "schema metadata",
    );
    insert_header(
        metadata,
        WENDAO_REPO_DOC_COVERAGE_REPO_HEADER,
        repo_id,
        "repo doc coverage repo metadata",
    );
    if let Some(module_id) = module_id {
        insert_header(
            metadata,
            WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
            module_id,
            "repo doc coverage module metadata",
        );
    }
}

pub(super) fn populate_repo_overview_headers(metadata: &mut MetadataMap, repo_id: &str) {
    populate_schema_headers(metadata);
    insert_header(
        metadata,
        WENDAO_REPO_OVERVIEW_REPO_HEADER,
        repo_id,
        "repo overview repo metadata",
    );
}

pub(super) fn populate_repo_index_headers(
    metadata: &mut MetadataMap,
    repo_id: Option<&str>,
    refresh: bool,
) {
    populate_schema_headers(metadata);
    if let Some(repo_id) = repo_id {
        insert_header(
            metadata,
            WENDAO_REPO_INDEX_REPO_HEADER,
            repo_id,
            "repo index repo metadata",
        );
    }
    insert_header(
        metadata,
        WENDAO_REPO_INDEX_REFRESH_HEADER,
        if refresh { "true" } else { "false" },
        "repo index refresh metadata",
    );
    insert_header(
        metadata,
        WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
        "repo-index-test-request",
        "repo index request id metadata",
    );
}

pub(super) fn populate_repo_index_status_headers(
    metadata: &mut MetadataMap,
    repo_id: Option<&str>,
) {
    populate_schema_headers(metadata);
    if let Some(repo_id) = repo_id {
        insert_header(
            metadata,
            WENDAO_REPO_INDEX_STATUS_REPO_HEADER,
            repo_id,
            "repo index status repo metadata",
        );
    }
}

pub(super) fn populate_repo_sync_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    mode: Option<&str>,
) {
    populate_schema_headers(metadata);
    insert_header(
        metadata,
        WENDAO_REPO_SYNC_REPO_HEADER,
        repo_id,
        "repo sync repo metadata",
    );
    if let Some(mode) = mode {
        insert_header(
            metadata,
            WENDAO_REPO_SYNC_MODE_HEADER,
            mode,
            "repo sync mode metadata",
        );
    }
}

pub(super) fn populate_repo_projected_page_index_tree_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
) {
    populate_schema_headers(metadata);
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
        repo_id,
        "repo projected page-index tree repo metadata",
    );
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
        page_id,
        "repo projected page-index tree page metadata",
    );
}

pub(super) fn populate_repo_projected_retrieval_context_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
    node_id: Option<&str>,
    related_limit: usize,
) {
    populate_schema_headers(metadata);
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
        repo_id,
        "repo projected retrieval context repo metadata",
    );
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
        page_id,
        "repo projected retrieval context page metadata",
    );
    if let Some(node_id) = node_id {
        insert_header(
            metadata,
            WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
            node_id,
            "repo projected retrieval context node metadata",
        );
    }
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
        &related_limit.to_string(),
        "repo projected retrieval context related limit metadata",
    );
}

pub(super) fn populate_refine_doc_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    entity_id: &str,
    user_hints_base64: Option<&str>,
) {
    populate_schema_headers(metadata);
    insert_header(
        metadata,
        WENDAO_REFINE_DOC_REPO_HEADER,
        repo_id,
        "refine doc repo metadata",
    );
    insert_header(
        metadata,
        WENDAO_REFINE_DOC_ENTITY_ID_HEADER,
        entity_id,
        "refine doc entity metadata",
    );
    if let Some(user_hints_base64) = user_hints_base64 {
        insert_header(
            metadata,
            WENDAO_REFINE_DOC_USER_HINTS_HEADER,
            user_hints_base64,
            "refine doc user hints metadata",
        );
    }
}
