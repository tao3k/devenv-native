//! Autocomplete route contract and metadata validation.

/// Canonical autocomplete prefix metadata header for Wendao Flight requests.
pub const WENDAO_AUTOCOMPLETE_PREFIX_HEADER: &str = "x-wendao-autocomplete-prefix";
/// Canonical autocomplete result-limit metadata header for Wendao Flight requests.
pub const WENDAO_AUTOCOMPLETE_LIMIT_HEADER: &str = "x-wendao-autocomplete-limit";
/// Stable route for the autocomplete contract.
pub const SEARCH_AUTOCOMPLETE_ROUTE: &str = "/search/autocomplete";

/// Validate the stable autocomplete request contract.
///
/// # Errors
///
/// Returns an error when the requested limit is zero or when the optional
/// prefix contains only whitespace.
pub fn validate_autocomplete_request(prefix: &str, limit: usize) -> Result<(), String> {
    if limit == 0 {
        return Err("autocomplete limit must be greater than zero".to_string());
    }
    if !prefix.is_empty() && prefix.trim().is_empty() {
        return Err("autocomplete prefix must not be blank".to_string());
    }
    Ok(())
}
