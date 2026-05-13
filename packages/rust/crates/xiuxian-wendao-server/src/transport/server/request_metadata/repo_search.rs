//! Repository-search Flight metadata validator.

use tonic::Status;
use tonic::metadata::MetadataMap;

use super::header_values::split_non_empty_header_values;
use crate::transport::query_contract::{
    RepoSearchRequest, WENDAO_REPO_SEARCH_FILENAME_FILTERS_HEADER,
    WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
    WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER, WENDAO_REPO_SEARCH_QUERY_HEADER,
    WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_REPO_SEARCH_TAG_FILTERS_HEADER,
    WENDAO_REPO_SEARCH_TITLE_FILTERS_HEADER, validate_repo_search_request,
};
use crate::transport::server::types::RepoSearchFlightRequest;

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
    validate_repo_search_request(RepoSearchRequest {
        query_text: query_text.as_str(),
        limit: parsed_limit,
        language_filters: &language_filter_values,
        path_prefixes: &path_prefix_values,
        title_filters: &title_filter_values,
        tag_filters: &tag_filter_values,
        filename_filters: &filename_filter_values,
    })
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
