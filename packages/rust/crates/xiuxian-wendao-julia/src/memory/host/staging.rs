//! Validation helpers for staging memory host request rows.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

pub(super) fn required_text(
    value: &str,
    field: &str,
    surface: &str,
) -> Result<String, RepoIntelligenceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(staging_error(
            surface,
            format!("field `{field}` must not be blank"),
        ));
    }
    Ok(trimmed.to_string())
}

pub(super) fn optional_text(value: Option<&str>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

pub(super) fn positive_u32_from_usize(
    value: usize,
    field: &str,
    surface: &str,
) -> Result<u32, RepoIntelligenceError> {
    let value = u32::try_from(value)
        .map_err(|_| staging_error(surface, format!("field `{field}` exceeds u32 range")))?;
    if value == 0 {
        return Err(staging_error(
            surface,
            format!("field `{field}` must be greater than zero"),
        ));
    }
    Ok(value)
}

pub(super) fn validate_probability(
    name: &str,
    value: f32,
    surface: &str,
) -> Result<(), RepoIntelligenceError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(staging_error(
            surface,
            format!("field `{name}` must be finite and in [0, 1]"),
        ));
    }
    Ok(())
}

pub(super) fn validate_finite(
    name: &str,
    value: f32,
    surface: &str,
) -> Result<(), RepoIntelligenceError> {
    if !value.is_finite() {
        return Err(staging_error(
            surface,
            format!("field `{name}` must be finite"),
        ));
    }
    Ok(())
}

pub(super) fn validate_non_negative_finite(
    name: &str,
    value: f32,
    surface: &str,
) -> Result<(), RepoIntelligenceError> {
    if !value.is_finite() || value < 0.0 {
        return Err(staging_error(
            surface,
            format!("field `{name}` must be finite and non-negative"),
        ));
    }
    Ok(())
}

pub(super) fn validate_embedding(
    name: &str,
    values: &[f32],
    surface: &str,
) -> Result<(), RepoIntelligenceError> {
    if values.is_empty() {
        return Err(staging_error(
            surface,
            format!("field `{name}` must not be empty"),
        ));
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(staging_error(
                surface,
                format!("field `{name}` contains non-finite value at index {index}"),
            ));
        }
    }
    Ok(())
}

pub(super) fn staging_error(surface: &str, message: impl Into<String>) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!("{surface} {}", message.into()),
    }
}
