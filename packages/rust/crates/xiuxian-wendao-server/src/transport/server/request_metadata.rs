use arrow_flight::{FlightDescriptor, Ticket};
use std::collections::HashSet;
use tonic::Status;
use tonic::metadata::MetadataMap;

use super::types::RepoSearchFlightRequest;
use crate::transport::query_contract::{
    DocumentExtractFlightRequest, DocumentExtractMode, SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE,
    SEARCH_REFERENCES_ROUTE, SEARCH_SYMBOLS_ROUTE, WENDAO_ANALYSIS_LINE_HEADER,
    WENDAO_ANALYSIS_PATH_HEADER, WENDAO_ANALYSIS_REPO_HEADER,
    WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER, WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER,
    WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER, WENDAO_AUTOCOMPLETE_LIMIT_HEADER,
    WENDAO_AUTOCOMPLETE_PREFIX_HEADER, WENDAO_DEFINITION_LINE_HEADER,
    WENDAO_DEFINITION_PATH_HEADER, WENDAO_DEFINITION_QUERY_HEADER,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER, WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER, WENDAO_DOCUMENT_EXTRACT_MODE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER, WENDAO_GRAPH_DIRECTION_HEADER,
    WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER, WENDAO_GRAPH_NODE_ID_HEADER,
    WENDAO_REFINE_DOC_ENTITY_ID_HEADER, WENDAO_REFINE_DOC_REPO_HEADER,
    WENDAO_REFINE_DOC_USER_HINTS_HEADER, WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
    WENDAO_REPO_DOC_COVERAGE_REPO_HEADER, WENDAO_REPO_INDEX_STATUS_REPO_HEADER,
    WENDAO_REPO_OVERVIEW_REPO_HEADER, WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER, WENDAO_REPO_SEARCH_FILENAME_FILTERS_HEADER,
    WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
    WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER, WENDAO_REPO_SEARCH_QUERY_HEADER,
    WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_REPO_SEARCH_TAG_FILTERS_HEADER,
    WENDAO_REPO_SEARCH_TITLE_FILTERS_HEADER, WENDAO_REPO_SYNC_MODE_HEADER,
    WENDAO_REPO_SYNC_REPO_HEADER, WENDAO_RERANK_DIMENSION_HEADER,
    WENDAO_RERANK_MIN_FINAL_SCORE_HEADER, WENDAO_RERANK_TOP_K_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
    WENDAO_SEARCH_INTENT_HEADER, WENDAO_SEARCH_LIMIT_HEADER, WENDAO_SEARCH_QUERY_HEADER,
    WENDAO_SEARCH_REPO_HEADER, WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER,
    WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER, WENDAO_SQL_QUERY_HEADER, WENDAO_VFS_PATH_HEADER,
    normalize_flight_route, validate_attachment_search_request, validate_autocomplete_request,
    validate_code_ast_analysis_request, validate_definition_request,
    validate_document_extract_request, validate_graph_neighbors_request,
    validate_markdown_analysis_request, validate_refine_doc_request,
    validate_repo_doc_coverage_request, validate_repo_index_request,
    validate_repo_index_status_request, validate_repo_overview_request,
    validate_repo_projected_page_index_tree_request, validate_repo_search_request,
    validate_repo_sync_request, validate_semantic_scope_request, validate_vfs_content_request,
    validate_vfs_resolve_request,
};

type AttachmentSearchMetadata = (String, usize, HashSet<String>, HashSet<String>, bool);
type RepoOverviewMetadata = String;
type RepoIndexMetadata = (Option<String>, bool, String);
type RepoIndexStatusMetadata = Option<String>;
type RepoSyncMetadata = (String, String);
type RepoDocCoverageMetadata = (String, Option<String>);
type RepoProjectedPageIndexTreeMetadata = (String, String);
type RefineDocMetadata = (String, String, Option<String>);

pub(crate) fn validate_schema_version(
    metadata: &MetadataMap,
    expected_schema_version: &str,
) -> Result<(), Status> {
    let schema_version = metadata
        .get(WENDAO_SCHEMA_VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if schema_version != expected_schema_version {
        return Err(Status::invalid_argument(format!(
            "unexpected schema version header: {schema_version}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_rerank_dimension_header(metadata: &MetadataMap) -> Result<usize, Status> {
    let dimension = metadata
        .get(WENDAO_RERANK_DIMENSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let parsed_dimension = dimension.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid rerank dimension header `{WENDAO_RERANK_DIMENSION_HEADER}`: {dimension}"
        ))
    })?;
    if parsed_dimension == 0 {
        return Err(Status::invalid_argument(format!(
            "rerank dimension header `{WENDAO_RERANK_DIMENSION_HEADER}` must be greater than zero"
        )));
    }
    Ok(parsed_dimension)
}

pub(crate) fn validate_rerank_top_k_header(
    metadata: &MetadataMap,
) -> Result<Option<usize>, Status> {
    let Some(raw_value) = metadata.get(WENDAO_RERANK_TOP_K_HEADER) else {
        return Ok(None);
    };
    let top_k = raw_value.to_str().unwrap_or_default();
    let parsed_top_k = top_k.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid rerank top_k header `{WENDAO_RERANK_TOP_K_HEADER}`: {top_k}"
        ))
    })?;
    if parsed_top_k == 0 {
        return Err(Status::invalid_argument(format!(
            "rerank top_k header `{WENDAO_RERANK_TOP_K_HEADER}` must be greater than zero"
        )));
    }
    Ok(Some(parsed_top_k))
}

pub(crate) fn validate_rerank_min_final_score_header(
    metadata: &MetadataMap,
) -> Result<Option<f64>, Status> {
    let Some(raw_value) = metadata.get(WENDAO_RERANK_MIN_FINAL_SCORE_HEADER) else {
        return Ok(None);
    };
    let min_final_score = raw_value.to_str().unwrap_or_default();
    let parsed_min_final_score = min_final_score.parse::<f64>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid rerank min_final_score header `{WENDAO_RERANK_MIN_FINAL_SCORE_HEADER}`: {min_final_score}"
        ))
    })?;
    if !parsed_min_final_score.is_finite() {
        return Err(Status::invalid_argument(format!(
            "rerank min_final_score header `{WENDAO_RERANK_MIN_FINAL_SCORE_HEADER}` must be finite"
        )));
    }
    if !(0.0..=1.0).contains(&parsed_min_final_score) {
        return Err(Status::invalid_argument(format!(
            "rerank min_final_score header `{WENDAO_RERANK_MIN_FINAL_SCORE_HEADER}` must stay within inclusive range [0.0, 1.0]"
        )));
    }
    Ok(Some(parsed_min_final_score))
}

pub(crate) fn validate_repo_search_request_metadata(
    metadata: &MetadataMap,
) -> Result<RepoSearchFlightRequest, Status> {
    let repo_id = metadata
        .get(WENDAO_REPO_SEARCH_REPO_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim()
        .to_string();
    if repo_id.is_empty() {
        return Err(Status::invalid_argument(format!(
            "repo search header `{WENDAO_REPO_SEARCH_REPO_HEADER}` must not be blank"
        )));
    }
    let query_text = metadata
        .get(WENDAO_REPO_SEARCH_QUERY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let limit = metadata
        .get(WENDAO_REPO_SEARCH_LIMIT_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let parsed_limit = limit.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid repo search limit header `{WENDAO_REPO_SEARCH_LIMIT_HEADER}`: {limit}"
        ))
    })?;
    let language_filter_values =
        split_non_empty_header_values(metadata, WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER);
    let path_prefix_values =
        split_non_empty_header_values(metadata, WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER);
    let title_filter_values =
        split_non_empty_header_values(metadata, WENDAO_REPO_SEARCH_TITLE_FILTERS_HEADER);
    let tag_filter_values =
        split_non_empty_header_values(metadata, WENDAO_REPO_SEARCH_TAG_FILTERS_HEADER);
    let filename_filter_values =
        split_non_empty_header_values(metadata, WENDAO_REPO_SEARCH_FILENAME_FILTERS_HEADER);
    validate_repo_search_request(
        query_text.as_str(),
        parsed_limit,
        &language_filter_values,
        &path_prefix_values,
        &title_filter_values,
        &tag_filter_values,
        &filename_filter_values,
    )
    .map_err(Status::invalid_argument)?;
    let language_filters = language_filter_values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::HashSet<_>>();
    let path_prefixes = path_prefix_values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::HashSet<_>>();
    let title_filters = title_filter_values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::HashSet<_>>();
    let tag_filters = tag_filter_values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::HashSet<_>>();
    let filename_filters = filename_filter_values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::HashSet<_>>();
    Ok(RepoSearchFlightRequest {
        repo_id,
        query_text,
        limit: parsed_limit,
        language_filters,
        path_prefixes,
        title_filters,
        tag_filters,
        filename_filters,
    })
}

fn split_non_empty_header_values(metadata: &MetadataMap, header: &'static str) -> Vec<String> {
    metadata
        .get(header)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim()
        .split(',')
        .filter(|value| !value.is_empty() || metadata.contains_key(header))
        .map(ToString::to_string)
        .collect()
}

pub(crate) fn validate_search_request_metadata(
    metadata: &MetadataMap,
) -> Result<(String, usize, Option<String>, Option<String>), Status> {
    let query_text = metadata
        .get(WENDAO_SEARCH_QUERY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let limit = metadata
        .get(WENDAO_SEARCH_LIMIT_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let parsed_limit = limit.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid search limit header `{WENDAO_SEARCH_LIMIT_HEADER}`: {limit}"
        ))
    })?;
    let intent = metadata
        .get(WENDAO_SEARCH_INTENT_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let repo_hint = metadata
        .get(WENDAO_SEARCH_REPO_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    validate_repo_search_request(query_text.as_str(), parsed_limit, &[], &[], &[], &[], &[])
        .map_err(Status::invalid_argument)?;
    Ok((query_text, parsed_limit, intent, repo_hint))
}

pub(crate) fn validate_repo_overview_request_metadata(
    metadata: &MetadataMap,
) -> Result<RepoOverviewMetadata, Status> {
    let repo_id = metadata
        .get(WENDAO_REPO_OVERVIEW_REPO_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    validate_repo_overview_request(repo_id).map_err(Status::invalid_argument)
}

pub(crate) fn validate_repo_index_status_request_metadata(
    metadata: &MetadataMap,
) -> RepoIndexStatusMetadata {
    let repo_id = metadata
        .get(WENDAO_REPO_INDEX_STATUS_REPO_HEADER)
        .and_then(|value| value.to_str().ok());
    validate_repo_index_status_request(repo_id)
}

pub(crate) fn validate_repo_index_request_metadata(
    metadata: &MetadataMap,
) -> Result<RepoIndexMetadata, Status> {
    let repo_id = metadata
        .get(crate::transport::query_contract::WENDAO_REPO_INDEX_REPO_HEADER)
        .and_then(|value| value.to_str().ok());
    let refresh = metadata
        .get(crate::transport::query_contract::WENDAO_REPO_INDEX_REFRESH_HEADER)
        .and_then(|value| value.to_str().ok());
    let request_id = metadata
        .get(crate::transport::query_contract::WENDAO_REPO_INDEX_REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    validate_repo_index_request(repo_id, refresh, request_id).map_err(Status::invalid_argument)
}

pub(crate) fn validate_repo_sync_request_metadata(
    metadata: &MetadataMap,
) -> Result<RepoSyncMetadata, Status> {
    let repo_id = metadata
        .get(WENDAO_REPO_SYNC_REPO_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let mode = metadata
        .get(WENDAO_REPO_SYNC_MODE_HEADER)
        .and_then(|value| value.to_str().ok());
    validate_repo_sync_request(repo_id, mode).map_err(Status::invalid_argument)
}

pub(crate) fn validate_definition_request_metadata(
    metadata: &MetadataMap,
) -> Result<(String, Option<String>, Option<usize>), Status> {
    let query_text = metadata
        .get(WENDAO_DEFINITION_QUERY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let source_path = metadata
        .get(WENDAO_DEFINITION_PATH_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let source_line = match metadata.get(WENDAO_DEFINITION_LINE_HEADER) {
        Some(raw_value) => {
            let source_line = raw_value.to_str().unwrap_or_default();
            Some(source_line.parse::<usize>().map_err(|_| {
                Status::invalid_argument(format!(
                    "invalid definition line header `{WENDAO_DEFINITION_LINE_HEADER}`: {source_line}"
                ))
            })?)
        }
        None => None,
    };
    validate_definition_request(query_text.as_str(), source_path.as_deref(), source_line)
        .map_err(Status::invalid_argument)?;
    Ok((query_text, source_path, source_line))
}

pub(crate) fn validate_autocomplete_request_metadata(
    metadata: &MetadataMap,
) -> Result<(String, usize), Status> {
    let prefix = metadata
        .get(WENDAO_AUTOCOMPLETE_PREFIX_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let limit = metadata
        .get(WENDAO_AUTOCOMPLETE_LIMIT_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let parsed_limit = limit.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid autocomplete limit header `{WENDAO_AUTOCOMPLETE_LIMIT_HEADER}`: {limit}"
        ))
    })?;
    validate_autocomplete_request(prefix.as_str(), parsed_limit)
        .map_err(Status::invalid_argument)?;
    Ok((prefix, parsed_limit))
}

pub(crate) fn validate_sql_request_metadata(metadata: &MetadataMap) -> Result<String, Status> {
    let query_text = metadata
        .get(WENDAO_SQL_QUERY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    if query_text.trim().is_empty() {
        return Err(Status::invalid_argument("SQL query text must not be blank"));
    }
    Ok(query_text)
}

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
        .map_err(Status::invalid_argument)
}

pub(crate) fn validate_document_extract_request_metadata(
    metadata: &MetadataMap,
) -> Result<DocumentExtractFlightRequest, Status> {
    let source_path = metadata
        .get(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    validate_document_extract_request(source_path.as_str()).map_err(Status::invalid_argument)?;
    let output_dir = metadata
        .get(WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let force = optional_document_extract_bool(
        metadata,
        WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
        "force",
        false,
    )?;
    let error_row = optional_document_extract_bool(
        metadata,
        WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
        "error_row",
        true,
    )?;
    let mode = metadata
        .get(WENDAO_DOCUMENT_EXTRACT_MODE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map_or(Ok(DocumentExtractMode::Sync), DocumentExtractMode::parse)
        .map_err(Status::invalid_argument)?;
    let wait_ms = optional_document_extract_u64(
        metadata,
        WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER,
        "wait_ms",
        0,
    )?;
    Ok(DocumentExtractFlightRequest {
        source_path,
        output_dir,
        force,
        error_row,
        mode,
        wait_ms,
    })
}

fn optional_document_extract_bool(
    metadata: &MetadataMap,
    header: &'static str,
    label: &str,
    default: bool,
) -> Result<bool, Status> {
    let Some(raw) = metadata.get(header).and_then(|value| value.to_str().ok()) else {
        return Ok(default);
    };
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(Status::invalid_argument(format!(
            "invalid document extract {label} header `{header}`"
        ))),
    }
}

pub(crate) fn validate_document_extract_status_request_metadata(
    metadata: &MetadataMap,
) -> Result<String, Status> {
    let job_id = metadata
        .get(WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim()
        .to_string();
    if job_id.is_empty() {
        return Err(Status::invalid_argument(
            "document extract job id must not be blank",
        ));
    }
    Ok(job_id)
}

fn optional_document_extract_u64(
    metadata: &MetadataMap,
    header: &'static str,
    label: &str,
    default: u64,
) -> Result<u64, Status> {
    let Some(raw) = metadata.get(header).and_then(|value| value.to_str().ok()) else {
        return Ok(default);
    };
    raw.trim().parse::<u64>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid document extract {label} header `{header}`: expected non-negative integer"
        ))
    })
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

pub(crate) fn validate_repo_doc_coverage_request_metadata(
    metadata: &MetadataMap,
) -> Result<RepoDocCoverageMetadata, Status> {
    let repo_id = metadata
        .get(WENDAO_REPO_DOC_COVERAGE_REPO_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let module_id = metadata
        .get(WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER)
        .and_then(|value| value.to_str().ok());
    validate_repo_doc_coverage_request(repo_id.as_str(), module_id)
        .map_err(Status::invalid_argument)
}

pub(crate) fn validate_repo_projected_page_index_tree_request_metadata(
    metadata: &MetadataMap,
) -> Result<RepoProjectedPageIndexTreeMetadata, Status> {
    let repo_id = metadata
        .get(WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let page_id = metadata
        .get(WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    validate_repo_projected_page_index_tree_request(repo_id.as_str(), page_id.as_str())
        .map_err(Status::invalid_argument)
}

pub(crate) fn validate_refine_doc_request_metadata(
    metadata: &MetadataMap,
) -> Result<RefineDocMetadata, Status> {
    let repo_id = metadata
        .get(WENDAO_REFINE_DOC_REPO_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let entity_id = metadata
        .get(WENDAO_REFINE_DOC_ENTITY_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let user_hints = metadata
        .get(WENDAO_REFINE_DOC_USER_HINTS_HEADER)
        .and_then(|value| value.to_str().ok());
    validate_refine_doc_request(repo_id.as_str(), entity_id.as_str(), user_hints)
        .map_err(Status::invalid_argument)
}

pub(crate) fn validate_attachment_search_request_metadata(
    metadata: &MetadataMap,
) -> Result<AttachmentSearchMetadata, Status> {
    let query_text = metadata
        .get(WENDAO_SEARCH_QUERY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let limit = metadata
        .get(WENDAO_SEARCH_LIMIT_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let parsed_limit = limit.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid search limit header `{WENDAO_SEARCH_LIMIT_HEADER}`: {limit}"
        ))
    })?;
    let ext_filter_values = metadata
        .get(WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim()
        .split(',')
        .filter(|value| {
            !value.is_empty() || metadata.contains_key(WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER)
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let kind_filter_values = metadata
        .get(WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim()
        .split(',')
        .filter(|value| {
            !value.is_empty() || metadata.contains_key(WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER)
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let case_sensitive = metadata
        .get(WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("false")
        .parse::<bool>()
        .map_err(|_| {
            Status::invalid_argument(format!(
                "invalid attachment-search case_sensitive header `{WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER}`"
            ))
        })?;
    validate_attachment_search_request(
        query_text.as_str(),
        parsed_limit,
        &ext_filter_values,
        &kind_filter_values,
    )
    .map_err(Status::invalid_argument)?;
    let ext_filters = ext_filter_values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::HashSet<_>>();
    let kind_filters = kind_filter_values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::HashSet<_>>();
    Ok((
        query_text,
        parsed_limit,
        ext_filters,
        kind_filters,
        case_sensitive,
    ))
}

pub(crate) fn descriptor_route(descriptor: &FlightDescriptor) -> Result<String, Status> {
    let actual_path = descriptor
        .path
        .iter()
        .map(|segment| String::from_utf8_lossy(segment.as_ref()).into_owned())
        .collect::<Vec<_>>();
    normalize_flight_route(format!("/{}", actual_path.join("/"))).map_err(Status::invalid_argument)
}

pub(crate) fn ticket_route(ticket: &Ticket) -> Result<String, Status> {
    let route = String::from_utf8(ticket.ticket.to_vec())
        .map_err(|error| Status::invalid_argument(format!("invalid ticket bytes: {error}")))?;
    normalize_flight_route(route).map_err(Status::invalid_argument)
}

pub(crate) fn join_sorted_set(values: &std::collections::HashSet<String>) -> String {
    let mut sorted = values.iter().cloned().collect::<Vec<_>>();
    sorted.sort();
    sorted.join(",")
}

pub(crate) fn is_search_family_route(route: &str) -> bool {
    matches!(
        route,
        SEARCH_INTENT_ROUTE
            | SEARCH_KNOWLEDGE_ROUTE
            | SEARCH_REFERENCES_ROUTE
            | SEARCH_SYMBOLS_ROUTE
    )
}
