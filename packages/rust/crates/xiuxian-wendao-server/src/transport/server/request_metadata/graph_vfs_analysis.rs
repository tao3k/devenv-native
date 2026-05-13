//! Graph, VFS, and analysis route metadata validators.

use tonic::Status;
use tonic::metadata::MetadataMap;

use super::header_values::split_non_empty_header_values;
use crate::transport::query_contract::{
    WENDAO_ANALYSIS_LINE_HEADER, WENDAO_ANALYSIS_PATH_HEADER, WENDAO_ANALYSIS_REPO_HEADER,
    WENDAO_GRAPH_DIRECTION_HEADER, WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER,
    WENDAO_GRAPH_NODE_ID_HEADER, WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER,
    WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER, WENDAO_VFS_PATH_HEADER,
    validate_code_ast_analysis_request, validate_graph_neighbors_request,
    validate_markdown_analysis_request, validate_semantic_scope_request,
    validate_vfs_content_request, validate_vfs_resolve_request,
};

pub(crate) fn validate_vfs_resolve_request_metadata(
    metadata: &MetadataMap,
) -> Result<String, Status> {
    let path = metadata
        .get(WENDAO_VFS_PATH_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    validate_vfs_resolve_request(path.as_str()).map_err(Status::invalid_argument)?;
    Ok(path)
}

pub(crate) fn validate_vfs_content_request_metadata(
    metadata: &MetadataMap,
) -> Result<String, Status> {
    let path = metadata
        .get(WENDAO_VFS_PATH_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    validate_vfs_content_request(path.as_str()).map_err(Status::invalid_argument)?;
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
    let hops = metadata
        .get(WENDAO_GRAPH_HOPS_HEADER)
        .and_then(|value| value.to_str().ok());
    let limit = metadata
        .get(WENDAO_GRAPH_LIMIT_HEADER)
        .and_then(|value| value.to_str().ok());
    let parsed_hops = match hops {
        Some(raw_value) => Some(raw_value.parse::<usize>().map_err(|_| {
            Status::invalid_argument(format!(
                "invalid graph neighbors hops header `{WENDAO_GRAPH_HOPS_HEADER}`: {raw_value}"
            ))
        })?),
        None => None,
    };
    let parsed_limit = match limit {
        Some(raw_value) => Some(raw_value.parse::<usize>().map_err(|_| {
            Status::invalid_argument(format!(
                "invalid graph neighbors limit header `{WENDAO_GRAPH_LIMIT_HEADER}`: {raw_value}"
            ))
        })?),
        None => None,
    };

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

pub(crate) fn validate_markdown_analysis_request_metadata(
    metadata: &MetadataMap,
) -> Result<String, Status> {
    let path = metadata
        .get(WENDAO_ANALYSIS_PATH_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    validate_markdown_analysis_request(path.as_str()).map_err(Status::invalid_argument)?;
    Ok(path)
}

pub(crate) fn validate_code_ast_analysis_request_metadata(
    metadata: &MetadataMap,
) -> Result<(String, String, Option<usize>), Status> {
    let path = metadata
        .get(WENDAO_ANALYSIS_PATH_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let repo_id = metadata
        .get(WENDAO_ANALYSIS_REPO_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let line_hint = match metadata.get(WENDAO_ANALYSIS_LINE_HEADER) {
        Some(raw_value) => {
            let line_hint = raw_value.to_str().unwrap_or_default();
            Some(line_hint.parse::<usize>().map_err(|_| {
                Status::invalid_argument(format!(
                    "invalid analysis line header `{WENDAO_ANALYSIS_LINE_HEADER}`: {line_hint}"
                ))
            })?)
        }
        None => None,
    };
    validate_code_ast_analysis_request(path.as_str(), repo_id.as_str(), line_hint)
        .map_err(Status::invalid_argument)?;
    Ok((path, repo_id, line_hint))
}

pub(crate) fn validate_semantic_scope_request_metadata(
    metadata: &MetadataMap,
) -> Result<crate::transport::query_contract::SemanticScopeFlightRequest, Status> {
    let task_id = metadata
        .get(WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER)
        .and_then(|value| value.to_str().ok());
    let object_ids =
        split_non_empty_header_values(metadata, WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER)
            .into_iter()
            .map(|value| value.trim().to_string())
            .collect::<Vec<_>>();
    validate_semantic_scope_request(task_id, &object_ids).map_err(Status::invalid_argument)
}
