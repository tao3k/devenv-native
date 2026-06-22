//! Julia parser-summary route identity contract.

const FILE_SUMMARY_TRANSPORT_KEY: &str = "file_summary";
const ROOT_SUMMARY_TRANSPORT_KEY: &str = "root_summary";

pub(crate) const JULIA_FILE_SUMMARY_ROUTE: &str = "/wendao/code-parser/julia/file-summary";
pub(crate) const JULIA_ROOT_SUMMARY_ROUTE: &str = "/wendao/code-parser/julia/root-summary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParserSummaryRouteKind {
    FileSummary,
    RootSummary,
}

impl ParserSummaryRouteKind {
    pub(crate) fn option_key(self) -> &'static str {
        match self {
            Self::FileSummary => FILE_SUMMARY_TRANSPORT_KEY,
            Self::RootSummary => ROOT_SUMMARY_TRANSPORT_KEY,
        }
    }

    pub(crate) fn route(self) -> &'static str {
        match self {
            Self::FileSummary => JULIA_FILE_SUMMARY_ROUTE,
            Self::RootSummary => JULIA_ROOT_SUMMARY_ROUTE,
        }
    }

    pub(crate) fn summary_kind(self) -> &'static str {
        match self {
            Self::FileSummary => "julia_file_summary",
            Self::RootSummary => "julia_root_summary",
        }
    }
}
