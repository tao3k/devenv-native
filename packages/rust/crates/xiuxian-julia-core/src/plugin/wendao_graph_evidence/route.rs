//! `WendaoGraph` evidence route helpers.

use super::names::WENDAO_GRAPH_LINK_EVIDENCE_ROUTE;
use xiuxian_wendao_runtime::transport::normalize_flight_route;

/// Resolve one route into the planned `WendaoGraph` `LinkGraph` evidence route.
///
/// # Errors
///
/// Returns an error when the route does not normalize to the planned
/// `WendaoGraph` evidence path.
pub fn wendao_graph_link_evidence_route(route: impl AsRef<str>) -> Result<&'static str, String> {
    let normalized = normalize_flight_route(route)?;
    if normalized == WENDAO_GRAPH_LINK_EVIDENCE_ROUTE {
        Ok(WENDAO_GRAPH_LINK_EVIDENCE_ROUTE)
    } else {
        Err(format!(
            "unsupported WendaoGraph evidence Flight route `{normalized}`"
        ))
    }
}

/// Return whether one route belongs to the `WendaoGraph` evidence contract.
#[must_use]
pub fn is_wendao_graph_link_evidence_route(route: impl AsRef<str>) -> bool {
    wendao_graph_link_evidence_route(route).is_ok()
}
