//! Route metadata for the Julia graph-structural Flight contract.

//! Route resolution for the `WendaoSearch.jl` graph-structural contract.

use xiuxian_wendao_runtime::transport::normalize_flight_route;

use super::columns::{
    GRAPH_STRUCTURAL_FILTER_REQUEST_COLUMNS, GRAPH_STRUCTURAL_FILTER_RESPONSE_COLUMNS,
    GRAPH_STRUCTURAL_FILTER_ROUTE, GRAPH_STRUCTURAL_RERANK_REQUEST_COLUMNS,
    GRAPH_STRUCTURAL_RERANK_RESPONSE_COLUMNS, GRAPH_STRUCTURAL_RERANK_ROUTE,
    JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION,
};

/// Stable graph-structural exchange route kind owned by the Julia plugin crate.
/// Stable graph-structural exchange route kind owned by the Julia plugin crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphStructuralRouteKind {
    /// Soft-score structural rerank lane.
    StructuralRerank,
    /// Hard-gate constraint-filter lane.
    ConstraintFilter,
}

impl GraphStructuralRouteKind {
    /// Return the canonical route path for this graph-structural exchange kind.
    #[must_use]
    pub fn route(self) -> &'static str {
        match self {
            Self::StructuralRerank => GRAPH_STRUCTURAL_RERANK_ROUTE,
            Self::ConstraintFilter => GRAPH_STRUCTURAL_FILTER_ROUTE,
        }
    }

    /// Return the staged schema version for this graph-structural exchange kind.
    #[must_use]
    pub fn schema_version(self) -> &'static str {
        match self {
            Self::StructuralRerank | Self::ConstraintFilter => {
                JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION
            }
        }
    }

    /// Return the capability-manifest variant tag for this exchange kind.
    #[must_use]
    pub fn capability_variant(self) -> &'static str {
        match self {
            Self::StructuralRerank => "structural_rerank",
            Self::ConstraintFilter => "constraint_filter",
        }
    }

    /// Return the canonical request columns for this graph-structural exchange kind.
    #[must_use]
    pub fn request_columns(self) -> &'static [&'static str] {
        match self {
            Self::StructuralRerank => &GRAPH_STRUCTURAL_RERANK_REQUEST_COLUMNS,
            Self::ConstraintFilter => &GRAPH_STRUCTURAL_FILTER_REQUEST_COLUMNS,
        }
    }

    /// Return the canonical response columns for this graph-structural exchange kind.
    #[must_use]
    pub fn response_columns(self) -> &'static [&'static str] {
        match self {
            Self::StructuralRerank => &GRAPH_STRUCTURAL_RERANK_RESPONSE_COLUMNS,
            Self::ConstraintFilter => &GRAPH_STRUCTURAL_FILTER_RESPONSE_COLUMNS,
        }
    }
}

/// Resolve one route into the staged graph-structural exchange kind.
///
/// # Errors
///
/// Returns an error when the route does not normalize into one of the staged
/// graph-structural exchange paths.
pub fn graph_structural_route_kind(
    route: impl AsRef<str>,
) -> Result<GraphStructuralRouteKind, String> {
    let normalized = normalize_flight_route(route)?;
    match normalized.as_str() {
        GRAPH_STRUCTURAL_RERANK_ROUTE => Ok(GraphStructuralRouteKind::StructuralRerank),
        GRAPH_STRUCTURAL_FILTER_ROUTE => Ok(GraphStructuralRouteKind::ConstraintFilter),
        _ => Err(format!(
            "unsupported graph-structural Flight route `{normalized}`"
        )),
    }
}

/// Return whether one route belongs to the staged graph-structural exchange family.
#[must_use]
pub fn is_graph_structural_route(route: impl AsRef<str>) -> bool {
    graph_structural_route_kind(route).is_ok()
}
