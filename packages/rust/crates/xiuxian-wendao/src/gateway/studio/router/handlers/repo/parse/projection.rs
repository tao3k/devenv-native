use crate::analyzers::{ProjectedGapKind, ProjectionPageKind};
use crate::gateway::studio::router::StudioApiError;

pub(crate) fn parse_projection_page_kind(
    kind: Option<&str>,
) -> Result<Option<ProjectionPageKind>, StudioApiError> {
    match kind.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some("reference") => Ok(Some(ProjectionPageKind::Reference)),
        Some("how_to") => Ok(Some(ProjectionPageKind::HowTo)),
        Some("tutorial") => Ok(Some(ProjectionPageKind::Tutorial)),
        Some("explanation") => Ok(Some(ProjectionPageKind::Explanation)),
        Some(other) => Err(StudioApiError::bad_request(
            "INVALID_KIND",
            format!("unsupported projected page kind `{other}`"),
        )),
    }
}

pub(crate) fn required_projection_page_kind(
    kind: Option<&str>,
) -> Result<ProjectionPageKind, StudioApiError> {
    parse_projection_page_kind(kind)?
        .ok_or_else(|| StudioApiError::bad_request("MISSING_KIND", "`kind` is required"))
}

pub(crate) fn parse_projected_gap_kind(
    kind: Option<&str>,
) -> Result<Option<ProjectedGapKind>, StudioApiError> {
    match kind.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some("module_reference_without_documentation") => {
            Ok(Some(ProjectedGapKind::ModuleReferenceWithoutDocumentation))
        }
        Some("symbol_reference_without_documentation") => {
            Ok(Some(ProjectedGapKind::SymbolReferenceWithoutDocumentation))
        }
        Some("symbol_reference_unverified") => {
            Ok(Some(ProjectedGapKind::SymbolReferenceUnverified))
        }
        Some("example_how_to_without_anchor" | "example_howto_without_anchor") => {
            Ok(Some(ProjectedGapKind::ExampleHowToWithoutAnchor))
        }
        Some("documentation_page_without_anchor") => {
            Ok(Some(ProjectedGapKind::DocumentationPageWithoutAnchor))
        }
        Some(other) => Err(StudioApiError::bad_request(
            "INVALID_GAP_KIND",
            format!("unsupported projected gap kind `{other}`"),
        )),
    }
}
