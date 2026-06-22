//! Legacy Microsoft Office attachment extraction through Rust parsers.

mod doc;
mod extract;
mod format;
mod markdown;
mod metrics;
mod panic_guard;
mod ppt;
mod types;
mod xls;

pub use extract::extract_legacy_office;
pub use format::{LegacyOfficeFormat, is_supported_legacy_office_path, legacy_office_format};
pub use metrics::{LegacyOfficeQualityMetrics, legacy_office_quality_metrics};
pub use types::LegacyOfficeExtraction;
