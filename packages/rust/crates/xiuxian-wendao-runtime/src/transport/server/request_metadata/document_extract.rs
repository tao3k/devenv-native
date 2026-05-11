use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::query_contract::{
    DocumentExtractFlightRequest, DocumentExtractMode, WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER, WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER,
    WENDAO_DOCUMENT_EXTRACT_MODE_HEADER, WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER, normalize_document_extract_profile,
    validate_document_extract_request,
};

pub(crate) fn validate_document_extract_request_metadata(
    metadata: &MetadataMap,
) -> Result<DocumentExtractFlightRequest, Status> {
    let source_path = header_string(metadata, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER);
    validate_document_extract_request(source_path.as_str()).map_err(Status::invalid_argument)?;
    Ok(DocumentExtractFlightRequest {
        source_path,
        output_dir: header_string(metadata, WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER),
        force: optional_document_extract_bool(
            metadata,
            WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
            "force",
            false,
        )?,
        error_row: optional_document_extract_bool(
            metadata,
            WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
            "error_row",
            true,
        )?,
        profile: document_extract_profile(metadata)?,
        mode: document_extract_mode(metadata)?,
        wait_ms: optional_document_extract_u64(
            metadata,
            WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER,
            "wait_ms",
            0,
        )?,
    })
}

pub(crate) fn validate_document_extract_status_request_metadata(
    metadata: &MetadataMap,
) -> Result<String, Status> {
    let job_id = header_string(metadata, WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER)
        .trim()
        .to_string();
    if job_id.is_empty() {
        return Err(Status::invalid_argument(
            "document extract job id must not be blank",
        ));
    }
    Ok(job_id)
}

fn document_extract_profile(metadata: &MetadataMap) -> Result<String, Status> {
    metadata
        .get(WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map_or(Ok("full"), normalize_document_extract_profile)
        .map(str::to_string)
        .map_err(Status::invalid_argument)
}

fn document_extract_mode(metadata: &MetadataMap) -> Result<DocumentExtractMode, Status> {
    metadata
        .get(WENDAO_DOCUMENT_EXTRACT_MODE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map_or(Ok(DocumentExtractMode::Sync), DocumentExtractMode::parse)
        .map_err(Status::invalid_argument)
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

fn header_string(metadata: &MetadataMap, header: &'static str) -> String {
    metadata
        .get(header)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
