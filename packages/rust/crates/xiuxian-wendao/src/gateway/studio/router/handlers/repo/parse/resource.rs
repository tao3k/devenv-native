use crate::gateway::studio::router::StudioApiError;

pub(crate) fn required_page_id(page_id: Option<&str>) -> Result<String, StudioApiError> {
    page_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| StudioApiError::bad_request("MISSING_PAGE_ID", "`page_id` is required"))
}

pub(crate) fn required_gap_id(gap_id: Option<&str>) -> Result<String, StudioApiError> {
    gap_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| StudioApiError::bad_request("MISSING_GAP_ID", "`gap_id` is required"))
}

pub(crate) fn required_node_id(node_id: Option<&str>) -> Result<String, StudioApiError> {
    node_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| StudioApiError::bad_request("MISSING_NODE_ID", "`node_id` is required"))
}
