//! Modelica parser-summary route identity contract.

const FILE_SUMMARY_TRANSPORT_KEY: &str = "file_summary";

pub(crate) const MODELICA_FILE_SUMMARY_ROUTE: &str = "/wendao/code-parser/modelica/file-summary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParserSummaryRouteKind {
    FileSummary,
}

impl ParserSummaryRouteKind {
    pub(crate) fn option_key(self) -> &'static str {
        match self {
            Self::FileSummary => FILE_SUMMARY_TRANSPORT_KEY,
        }
    }

    pub(crate) fn route(self) -> &'static str {
        match self {
            Self::FileSummary => MODELICA_FILE_SUMMARY_ROUTE,
        }
    }
}
