//! Legacy Office extraction DTOs.

use super::LegacyOfficeFormat;
use super::metrics::LegacyOfficeQualityMetrics;

/// Parsed legacy Office content ready for Wendao resource-row projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyOfficeExtraction {
    /// Source file format selected from the extension.
    pub format: LegacyOfficeFormat,
    /// Plain text extracted by the parser.
    pub text: String,
    /// Markdown projection for agent consumption.
    pub markdown: String,
    /// Parser-quality counters used by gateway reports and precision gates.
    pub quality_metrics: LegacyOfficeQualityMetrics,
}
