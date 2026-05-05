use crate::studio::router::StudioApiError;

pub(crate) fn required_search_query(query: Option<&str>) -> Result<String, StudioApiError> {
    query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| StudioApiError::bad_request("MISSING_QUERY", "`query` is required"))
}

pub(crate) fn optional_search_filter(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn required_import_search_filters(
    package: Option<&str>,
    module: Option<&str>,
) -> Result<(Option<String>, Option<String>), StudioApiError> {
    let package = optional_search_filter(package);
    let module = optional_search_filter(module);
    if package.is_none() && module.is_none() {
        return Err(StudioApiError::bad_request(
            "MISSING_IMPORT_FILTER",
            "at least one of `package` or `module` is required",
        ));
    }
    Ok((package, module))
}
