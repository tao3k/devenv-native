//! Core schema and rerank metadata validators.

use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::query_contract::{
    WENDAO_RERANK_DIMENSION_HEADER, WENDAO_RERANK_MIN_FINAL_SCORE_HEADER,
    WENDAO_RERANK_TOP_K_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
};

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
