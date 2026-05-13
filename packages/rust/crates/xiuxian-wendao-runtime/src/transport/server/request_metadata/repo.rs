use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::query_contract::{
    RepoProjectedRetrievalContextInput, RepoProjectedRetrievalContextNodeId,
    WENDAO_REFINE_DOC_ENTITY_ID_HEADER, WENDAO_REFINE_DOC_REPO_HEADER,
    WENDAO_REFINE_DOC_USER_HINTS_HEADER, WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
    WENDAO_REPO_DOC_COVERAGE_REPO_HEADER, WENDAO_REPO_INDEX_REFRESH_HEADER,
    WENDAO_REPO_INDEX_REPO_HEADER, WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
    WENDAO_REPO_INDEX_STATUS_REPO_HEADER, WENDAO_REPO_OVERVIEW_REPO_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER, WENDAO_REPO_SYNC_MODE_HEADER,
    WENDAO_REPO_SYNC_REPO_HEADER, validate_refine_doc_request, validate_repo_doc_coverage_request,
    validate_repo_index_request, validate_repo_index_status_request,
    validate_repo_overview_request, validate_repo_projected_page_index_tree_request,
    validate_repo_projected_retrieval_context_request, validate_repo_sync_request,
};

type RepoOverviewMetadata = String;
type RepoIndexMetadata = (Option<String>, bool, String);
type RepoIndexStatusMetadata = Option<String>;
type RepoSyncMetadata = (String, String);
type RepoDocCoverageMetadata = (String, Option<String>);
type RepoProjectedPageIndexTreeMetadata = (String, String);
type RepoProjectedRetrievalContextMetadata = (String, String, Option<String>, usize);
type RefineDocMetadata = (String, String, Option<String>);

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
        .get(WENDAO_REPO_INDEX_REPO_HEADER)
        .and_then(|value| value.to_str().ok());
    let refresh = metadata
        .get(WENDAO_REPO_INDEX_REFRESH_HEADER)
        .and_then(|value| value.to_str().ok());
    let request_id = metadata
        .get(WENDAO_REPO_INDEX_REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    validate_repo_index_request(repo_id, refresh, request_id)
        .map(|request| (request.repo_id, request.refresh, request.request_id))
        .map_err(Status::invalid_argument)
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
    validate_repo_sync_request(repo_id, mode)
        .map(|request| (request.repo_id, request.mode.as_str().to_string()))
        .map_err(Status::invalid_argument)
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
        .map(|request| (request.repo_id, request.module_id))
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
        .map(|request| (request.repo_id, request.page_id))
        .map_err(Status::invalid_argument)
}

pub(crate) fn validate_repo_projected_retrieval_context_request_metadata(
    metadata: &MetadataMap,
) -> Result<RepoProjectedRetrievalContextMetadata, Status> {
    let repo_id = header_string(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
    );
    let page_id = header_string(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    );
    let node_id = metadata
        .get(WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER)
        .and_then(|value| value.to_str().ok());
    let related_limit = optional_usize_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
        "repo projected retrieval-context related-limit",
    )?;
    validate_repo_projected_retrieval_context_request(RepoProjectedRetrievalContextInput {
        repo_id: repo_id.as_str(),
        page_id: page_id.as_str(),
        node_id,
        related_limit,
    })
    .map(|request| {
        (
            request.repo_id.into_string(),
            request.page_id.into_string(),
            request
                .node_id
                .map(RepoProjectedRetrievalContextNodeId::into_string),
            request.related_limit,
        )
    })
    .map_err(Status::invalid_argument)
}

pub(crate) fn validate_refine_doc_request_metadata(
    metadata: &MetadataMap,
) -> Result<RefineDocMetadata, Status> {
    let repo_id = header_string(metadata, WENDAO_REFINE_DOC_REPO_HEADER);
    let entity_id = header_string(metadata, WENDAO_REFINE_DOC_ENTITY_ID_HEADER);
    let user_hints = metadata
        .get(WENDAO_REFINE_DOC_USER_HINTS_HEADER)
        .and_then(|value| value.to_str().ok());
    validate_refine_doc_request(repo_id.as_str(), entity_id.as_str(), user_hints)
        .map(|request| (request.repo_id, request.entity_id, request.user_hints))
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
