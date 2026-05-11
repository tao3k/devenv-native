use std::collections::HashSet;

use tonic::Status;
use tonic::metadata::MetadataMap;

use super::routing::split_non_empty_header_values;
use crate::transport::query_contract::{
    RepoSearchRequest, WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER,
    WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER, WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER,
    WENDAO_AUTOCOMPLETE_LIMIT_HEADER, WENDAO_AUTOCOMPLETE_PREFIX_HEADER,
    WENDAO_DEFINITION_LINE_HEADER, WENDAO_DEFINITION_PATH_HEADER, WENDAO_DEFINITION_QUERY_HEADER,
    WENDAO_REPO_SEARCH_FILENAME_FILTERS_HEADER, WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER,
    WENDAO_REPO_SEARCH_LIMIT_HEADER, WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER,
    WENDAO_REPO_SEARCH_QUERY_HEADER, WENDAO_REPO_SEARCH_REPO_HEADER,
    WENDAO_REPO_SEARCH_TAG_FILTERS_HEADER, WENDAO_REPO_SEARCH_TITLE_FILTERS_HEADER,
    WENDAO_SEARCH_INTENT_HEADER, WENDAO_SEARCH_LIMIT_HEADER, WENDAO_SEARCH_QUERY_HEADER,
    WENDAO_SEARCH_REPO_HEADER, validate_attachment_search_request, validate_autocomplete_request,
    validate_definition_request, validate_repo_search_request,
};

use crate::transport::server::types::RepoSearchFlightRequest;

type AttachmentSearchMetadata = (String, usize, HashSet<String>, HashSet<String>, bool);

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
    let parsed_limit = parse_usize_header(limit, WENDAO_REPO_SEARCH_LIMIT_HEADER, "repo search")?;
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
    Ok(RepoSearchFlightRequest {
        repo_id,
        query_text,
        limit: parsed_limit,
        language_filters: normalized_filter_set(language_filter_values),
        path_prefixes: normalized_filter_set(path_prefix_values),
        title_filters: normalized_filter_set(title_filter_values),
        tag_filters: normalized_filter_set(tag_filter_values),
        filename_filters: normalized_filter_set(filename_filter_values),
    })
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
    let parsed_limit = parse_usize_header(limit, WENDAO_SEARCH_LIMIT_HEADER, "search")?;
    let intent = optional_trimmed_header(metadata, WENDAO_SEARCH_INTENT_HEADER);
    let repo_hint = optional_trimmed_header(metadata, WENDAO_SEARCH_REPO_HEADER);
    validate_repo_search_request(RepoSearchRequest {
        query_text: query_text.as_str(),
        limit: parsed_limit,
        language_filters: &[],
        path_prefixes: &[],
        title_filters: &[],
        tag_filters: &[],
        filename_filters: &[],
    })
    .map_err(Status::invalid_argument)?;
    Ok((query_text, parsed_limit, intent, repo_hint))
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
    let source_line =
        optional_usize_header(metadata, WENDAO_DEFINITION_LINE_HEADER, "definition line")?;
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
    let parsed_limit = parse_usize_header(limit, WENDAO_AUTOCOMPLETE_LIMIT_HEADER, "autocomplete")?;
    validate_autocomplete_request(prefix.as_str(), parsed_limit)
        .map_err(Status::invalid_argument)?;
    Ok((prefix, parsed_limit))
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
    let parsed_limit = parse_usize_header(limit, WENDAO_SEARCH_LIMIT_HEADER, "search")?;
    let ext_filter_values =
        split_non_empty_header_values(metadata, WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER);
    let kind_filter_values =
        split_non_empty_header_values(metadata, WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER);
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
    Ok((
        query_text,
        parsed_limit,
        normalized_filter_set(ext_filter_values),
        normalized_filter_set(kind_filter_values),
        case_sensitive,
    ))
}

fn normalized_filter_set(values: Vec<String>) -> HashSet<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn optional_trimmed_header(metadata: &MetadataMap, header: &'static str) -> Option<String> {
    metadata
        .get(header)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn parse_usize_header(raw: &str, header: &'static str, label: &str) -> Result<usize, Status> {
    raw.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!("invalid {label} limit header `{header}`: {raw}"))
    })
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
