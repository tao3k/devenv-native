use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

pub(super) fn normalize_non_blank(
    value: impl AsRef<str>,
    field_name: &str,
) -> Result<String, RepoIntelligenceError> {
    let normalized = value.as_ref().trim().to_string();
    if normalized.is_empty() {
        return Err(graph_structural_projection_error(format!(
            "{field_name} must not be blank"
        )));
    }
    Ok(normalized)
}

pub(super) fn normalize_pair_endpoint_ids(
    left_id: String,
    right_id: String,
) -> Result<(String, String), RepoIntelligenceError> {
    let left_id = normalize_non_blank(left_id, "pair left id")?;
    let right_id = normalize_non_blank(right_id, "pair right id")?;
    if left_id == right_id {
        return Err(graph_structural_projection_error(
            "pair endpoints must not resolve to the same id",
        ));
    }
    Ok((left_id, right_id))
}

pub(super) fn normalize_string_list(
    values: Vec<String>,
    field_name: &str,
    allow_empty_list: bool,
) -> Result<Vec<String>, RepoIntelligenceError> {
    let mut normalized = Vec::with_capacity(values.len());
    for (index, value) in values.into_iter().enumerate() {
        let normalized_value = value.trim().to_string();
        if normalized_value.is_empty() {
            return Err(graph_structural_projection_error(format!(
                "{field_name} item {index} must not be blank"
            )));
        }
        normalized.push(normalized_value);
    }
    if !allow_empty_list && normalized.is_empty() {
        return Err(graph_structural_projection_error(format!(
            "{field_name} must contain at least one item"
        )));
    }
    Ok(normalized)
}

pub(super) fn normalize_non_negative_score(
    value: f64,
    field_name: &str,
) -> Result<f64, RepoIntelligenceError> {
    if !value.is_finite() {
        return Err(graph_structural_projection_error(format!(
            "{field_name} must be finite; found {value}"
        )));
    }
    if value < 0.0 {
        return Err(graph_structural_projection_error(format!(
            "{field_name} must be non-negative; found {value}"
        )));
    }
    Ok(value)
}

pub(super) fn binary_plane_score(matched: bool) -> f64 {
    if matched { 1.0 } else { 0.0 }
}

pub(super) fn stable_pair_candidate_id(left_id: &str, right_id: &str) -> String {
    if left_id <= right_id {
        format!("pair:{left_id}:{right_id}")
    } else {
        format!("pair:{right_id}:{left_id}")
    }
}

pub(super) fn graph_structural_projection_error(
    detail: impl Into<String>,
) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "invalid Julia graph-structural projection: {}",
            detail.into()
        ),
    }
}
