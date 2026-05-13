//! Generic search, definition, autocomplete, and SQL metadata validators.

use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::query_contract::{
    RepoSearchRequest, WENDAO_AUTOCOMPLETE_LIMIT_HEADER, WENDAO_AUTOCOMPLETE_PREFIX_HEADER,
    WENDAO_DEFINITION_LINE_HEADER, WENDAO_DEFINITION_PATH_HEADER, WENDAO_DEFINITION_QUERY_HEADER,
    WENDAO_SEARCH_INTENT_HEADER, WENDAO_SEARCH_LIMIT_HEADER, WENDAO_SEARCH_QUERY_HEADER,
    WENDAO_SEARCH_REPO_HEADER, WENDAO_SQL_QUERY_HEADER, validate_autocomplete_request,
    validate_definition_request, validate_repo_search_request,
};

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
