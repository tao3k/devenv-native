//! Document-extract metadata validators.

use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::query_contract::{
    DocumentExtractFlightRequest, DocumentExtractMode, WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER, WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER,
    WENDAO_DOCUMENT_EXTRACT_MODE_HEADER, WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER, WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER,
    decode_document_extract_source_path_utf8_hex, normalize_document_extract_profile,
    validate_document_extract_request,
};

pub(crate) fn validate_document_extract_request_metadata(
    metadata: &MetadataMap,
) -> Result<DocumentExtractFlightRequest, Status> {
    let source_path = document_extract_source_path(metadata)?;
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
    let profile = metadata
        .get(WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map_or(Ok("full"), normalize_document_extract_profile)
        .map_err(Status::invalid_argument)?
        .to_string();
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
        profile,
        mode,
        wait_ms,
    })
}

fn document_extract_source_path(metadata: &MetadataMap) -> Result<String, Status> {
    if let Some(encoded) = metadata
        .get(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
    {
        return decode_document_extract_source_path_utf8_hex(encoded)
            .map_err(Status::invalid_argument);
    }
    Ok(metadata
        .get(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string())
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
