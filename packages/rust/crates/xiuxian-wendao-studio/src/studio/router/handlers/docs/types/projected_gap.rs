use serde::Deserialize;

/// Query parameters for docs-facing projected gap lookup.
#[derive(Debug, Deserialize)]
pub struct DocsProjectedGapReportApiQuery {
    /// The repository identifier.
    pub(crate) repo: Option<String>,
}
