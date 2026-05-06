//! Attachment-search metadata validator.

use std::collections::HashSet;

use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::query_contract::{
    WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER, WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER,
    WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER, WENDAO_SEARCH_LIMIT_HEADER,
    WENDAO_SEARCH_QUERY_HEADER, validate_attachment_search_request,
};

type AttachmentSearchMetadata = (String, usize, HashSet<String>, HashSet<String>, bool);

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
