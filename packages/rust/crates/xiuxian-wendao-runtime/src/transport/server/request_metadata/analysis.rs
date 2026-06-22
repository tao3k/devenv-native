use tonic::Status;
use tonic::metadata::MetadataMap;

#[cfg(feature = "transport")]
use crate::transport::query_contract::validate_sql_query_request;
use crate::transport::query_contract::{
    WENDAO_ANALYSIS_PATH_HEADER, WENDAO_GRAPH_DIRECTION_HEADER, WENDAO_GRAPH_HOPS_HEADER,
    WENDAO_GRAPH_LIMIT_HEADER, WENDAO_GRAPH_NODE_ID_HEADER, WENDAO_SQL_QUERY_HEADER,
    WENDAO_VFS_PATH_HEADER, validate_graph_neighbors_request, validate_markdown_analysis_request,
    validate_vfs_content_request, validate_vfs_resolve_request,
};

pub(crate) fn validate_sql_request_metadata(metadata: &MetadataMap) -> Result<String, Status> {
    let query_text = header_string(metadata, WENDAO_SQL_QUERY_HEADER);
    #[cfg(feature = "transport")]
    validate_sql_query_request(query_text.as_str()).map_err(Status::invalid_argument)?;
    Ok(query_text)
}

pub(crate) fn validate_vfs_resolve_request_metadata(
    metadata: &MetadataMap,
) -> Result<String, Status> {
    let path = header_string(metadata, WENDAO_VFS_PATH_HEADER);
    validate_vfs_resolve_request(path.as_str()).map_err(Status::invalid_argument)?;
    Ok(path)
}

pub(crate) fn validate_vfs_content_request_metadata(
    metadata: &MetadataMap,
) -> Result<String, Status> {
    let path = header_string(metadata, WENDAO_VFS_PATH_HEADER);
    validate_vfs_content_request(path.as_str()).map_err(Status::invalid_argument)?;
    Ok(path)
}

pub(crate) fn validate_markdown_analysis_request_metadata(
    metadata: &MetadataMap,
) -> Result<String, Status> {
    let path = header_string(metadata, WENDAO_ANALYSIS_PATH_HEADER);
    validate_markdown_analysis_request(path.as_str()).map_err(Status::invalid_argument)?;
    Ok(path)
}

pub(crate) fn validate_graph_neighbors_request_metadata(
    metadata: &MetadataMap,
) -> Result<(String, String, usize, usize), Status> {
    let node_id = metadata
        .get(WENDAO_GRAPH_NODE_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let direction = metadata
        .get(WENDAO_GRAPH_DIRECTION_HEADER)
        .and_then(|value| value.to_str().ok());
    let parsed_hops =
        optional_usize_header(metadata, WENDAO_GRAPH_HOPS_HEADER, "graph neighbors hops")?;
    let parsed_limit =
        optional_usize_header(metadata, WENDAO_GRAPH_LIMIT_HEADER, "graph neighbors limit")?;

    validate_graph_neighbors_request(node_id, direction, parsed_hops, parsed_limit)
        .map(|request| {
            (
                request.node_id,
                request.direction,
                request.hops,
                request.limit,
            )
        })
        .map_err(Status::invalid_argument)
}

fn header_string(metadata: &MetadataMap, header: &'static str) -> String {
    metadata
        .get(header)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

fn optional_usize_header(
    metadata: &MetadataMap,
    header: &'static str,
    label: &str,
) -> Result<Option<usize>, Status> {
    metadata
        .get(header)
        .map(|raw_value| {
            let raw = raw_value.to_str().unwrap_or_default();
            raw.parse::<usize>().map_err(|_| {
                Status::invalid_argument(format!("invalid {label} header `{header}`: {raw}"))
            })
        })
        .transpose()
}
