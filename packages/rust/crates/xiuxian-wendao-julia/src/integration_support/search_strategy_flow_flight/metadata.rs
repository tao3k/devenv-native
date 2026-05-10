use tonic::metadata::MetadataMap;
use xiuxian_wendao_runtime::transport::{
    WENDAO_GRAPH_DIRECTION_HEADER, WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER,
    WENDAO_GRAPH_NODE_ID_HEADER, WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
    WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER, WENDAO_REPO_SEARCH_QUERY_HEADER,
    WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
};

pub(super) fn populate_repo_search_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    query_text: &str,
    limit: usize,
    path_prefix: &str,
) -> Result<(), String> {
    populate_schema_headers(metadata)?;
    insert_header(metadata, WENDAO_REPO_SEARCH_REPO_HEADER, repo_id)?;
    insert_header(metadata, WENDAO_REPO_SEARCH_QUERY_HEADER, query_text)?;
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_LIMIT_HEADER,
        &limit.to_string(),
    )?;
    if path_prefix.trim().is_empty() {
        return Ok(());
    }
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER,
        path_prefix,
    )
}

pub(super) fn populate_page_index_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
) -> Result<(), String> {
    populate_schema_headers(metadata)?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
        repo_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
        page_id,
    )
}

pub(super) fn populate_retrieval_context_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
    node_id: &str,
    related_limit: usize,
) -> Result<(), String> {
    populate_schema_headers(metadata)?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
        repo_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
        page_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
        node_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
        &related_limit.to_string(),
    )
}

pub(super) fn populate_graph_neighbors_headers(
    metadata: &mut MetadataMap,
    node_id: &str,
    direction: &str,
    hops: usize,
    limit: usize,
) -> Result<(), String> {
    populate_schema_headers(metadata)?;
    insert_header(metadata, WENDAO_GRAPH_NODE_ID_HEADER, node_id)?;
    insert_header(metadata, WENDAO_GRAPH_DIRECTION_HEADER, direction)?;
    insert_header(metadata, WENDAO_GRAPH_HOPS_HEADER, &hops.to_string())?;
    insert_header(metadata, WENDAO_GRAPH_LIMIT_HEADER, &limit.to_string())
}

fn populate_schema_headers(metadata: &mut MetadataMap) -> Result<(), String> {
    insert_header(metadata, WENDAO_SCHEMA_VERSION_HEADER, "v2")
}

fn insert_header(
    metadata: &mut MetadataMap,
    header: &'static str,
    value: &str,
) -> Result<(), String> {
    metadata.insert(
        header,
        value
            .parse()
            .map_err(|error| format!("invalid metadata value for `{header}`: {error}"))?,
    );
    Ok(())
}
