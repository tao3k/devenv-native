use std::path::Path;

use super::format::{LegacyOfficeFormat, legacy_office_format};
use super::markdown::legacy_office_markdown;
use super::metrics::legacy_office_quality_metrics;
use super::types::LegacyOfficeExtraction;
use super::{doc, ppt, xls};

/// Extracts legacy Office text using format-specific Rust parser paths.
///
/// # Errors
///
/// Returns an error when the path extension is unsupported, the source cannot
/// be parsed as the selected legacy Office format, or the parser returns empty
/// text.
pub fn extract_legacy_office(path: &Path) -> Result<LegacyOfficeExtraction, String> {
    let format = legacy_office_format(path).ok_or_else(|| {
        format!(
            "unsupported legacy Office source `{}`; expected .doc, .xls, or .ppt",
            path.display()
        )
    })?;
    let text = extract_text_by_format(path, format)?;
    let text = normalize_extracted_text(text.as_str());
    if text.is_empty() {
        return Err(format!(
            "legacy Office source `{}` produced no text",
            path.display()
        ));
    }
    let markdown = legacy_office_markdown(path, format, text.as_str())?;
    let quality_metrics = legacy_office_quality_metrics(format, text.as_str(), markdown.as_str());
    Ok(LegacyOfficeExtraction {
        format,
        text,
        markdown,
        quality_metrics,
    })
}

fn extract_text_by_format(path: &Path, format: LegacyOfficeFormat) -> Result<String, String> {
    match format {
        LegacyOfficeFormat::Doc => doc::extract_text(path),
        LegacyOfficeFormat::Xls => xls::extract_text(path),
        LegacyOfficeFormat::Ppt => ppt::extract_text(path),
    }
}

fn normalize_extracted_text(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
