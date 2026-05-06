//! Repository route metadata validators.

use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::query_contract::{
    WENDAO_REFINE_DOC_ENTITY_ID_HEADER, WENDAO_REFINE_DOC_REPO_HEADER,
    WENDAO_REFINE_DOC_USER_HINTS_HEADER, WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
    WENDAO_REPO_DOC_COVERAGE_REPO_HEADER, WENDAO_REPO_INDEX_REFRESH_HEADER,
    WENDAO_REPO_INDEX_REPO_HEADER, WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
    WENDAO_REPO_INDEX_STATUS_REPO_HEADER, WENDAO_REPO_OVERVIEW_REPO_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER, WENDAO_REPO_SYNC_MODE_HEADER,
    WENDAO_REPO_SYNC_REPO_HEADER, validate_refine_doc_request, validate_repo_doc_coverage_request,
    validate_repo_index_request, validate_repo_index_status_request,
    validate_repo_overview_request, validate_repo_projected_page_index_tree_request,
    validate_repo_sync_request,
};

type RepoOverviewMetadata = String;
type RepoIndexMetadata = (Option<String>, bool, String);
type RepoIndexStatusMetadata = Option<String>;
type RepoSyncMetadata = (String, String);
type RepoDocCoverageMetadata = (String, Option<String>);
type RepoProjectedPageIndexTreeMetadata = (String, String);
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
