//! Shared query-contract metadata helpers for Wendao Flight routes.

/// Canonical schema-version metadata header for Wendao Flight requests.
pub const WENDAO_SCHEMA_VERSION_HEADER: &str = "x-wendao-schema-version";

/// Normalize one route into the canonical leading-slash Flight form.
///
/// # Errors
///
/// Returns an error when the route resolves to no descriptor segments.
pub fn normalize_flight_route(route: impl AsRef<str>) -> Result<String, String> {
    let route = route.as_ref();
    let normalized = if route.starts_with('/') {
        route.to_string()
    } else {
        format!("/{route}")
    };
    if normalized.trim_matches('/').is_empty() {
        return Err(
            "Arrow Flight route must resolve to at least one descriptor segment".to_string(),
        );
    }
    Ok(normalized)
}

/// Convert one canonical Flight route into descriptor-path segments.
///
/// # Errors
///
/// Returns an error when the route resolves to no descriptor segments.
pub fn flight_descriptor_path(route: impl AsRef<str>) -> Result<Vec<String>, String> {
    let normalized = normalize_flight_route(route)?;
    let path = normalized
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if path.is_empty() {
        return Err(
            "Arrow Flight route must resolve to at least one descriptor segment".to_string(),
        );
    }
    Ok(path)
}
