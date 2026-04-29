use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use pdf_inspector::{
    DetectionConfig, LayoutComplexity, PdfOptions, PdfProcessResult, PdfType, ProcessMode,
    ScanStrategy, process_pdf_with_options,
};
use serde::{Deserialize, Serialize};

const FAST_RUST_CONFIDENCE_THRESHOLD: f32 = 0.90;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfInspectorAuditProfile {
    DetectFull,
    AnalyzeFull,
    Unsupported,
}

impl PdfInspectorAuditProfile {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DetectFull => "detect_full",
            Self::AnalyzeFull => "analyze_full",
            Self::Unsupported => "unsupported",
        }
    }

    fn process_mode(self) -> Option<ProcessMode> {
        match self {
            Self::DetectFull => Some(ProcessMode::DetectOnly),
            Self::AnalyzeFull => Some(ProcessMode::Analyze),
            Self::Unsupported => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfInspectorRoutingDecision {
    FastRustCandidate,
    HybridPageOcrCandidate,
    FullDoclingFallback,
    PreflightFailed,
    UnsupportedNonPdf,
}

impl PdfInspectorRoutingDecision {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FastRustCandidate => "fast_rust_candidate",
            Self::HybridPageOcrCandidate => "hybrid_page_ocr_candidate",
            Self::FullDoclingFallback => "full_docling_fallback",
            Self::PreflightFailed => "preflight_failed",
            Self::UnsupportedNonPdf => "unsupported_non_pdf",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfInspectorAuditRecord {
    pub source_path: String,
    pub file_size_bytes: Option<u64>,
    pub profile: String,
    pub pdf_type: Option<String>,
    pub page_count: Option<u32>,
    pub confidence: Option<f32>,
    pub pages_needing_ocr: Vec<u32>,
    pub is_complex: Option<bool>,
    pub pages_with_tables: Vec<u32>,
    pub pages_with_columns: Vec<u32>,
    pub has_encoding_issues: Option<bool>,
    pub elapsed_ms: f64,
    pub routing_decision: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfInspectorRoutingSignals {
    pub pdf_type: PdfInspectorPdfType,
    pub page_count: u32,
    pub confidence: f32,
    pub pages_needing_ocr: Vec<u32>,
    pub is_complex: bool,
    pub has_encoding_issues: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfInspectorPdfType {
    TextBased,
    Scanned,
    ImageBased,
    Mixed,
}

#[must_use]
pub fn audit_pdf_paths(paths: &[PathBuf]) -> Vec<PdfInspectorAuditRecord> {
    paths
        .iter()
        .flat_map(|path| audit_pdf_path(path.as_path()))
        .collect()
}

#[must_use]
pub fn audit_pdf_path(path: &Path) -> Vec<PdfInspectorAuditRecord> {
    let file_size_bytes = path.metadata().ok().map(|metadata| metadata.len());
    if !is_pdf_path(path) {
        return vec![unsupported_record(path, file_size_bytes)];
    }
    [
        PdfInspectorAuditProfile::DetectFull,
        PdfInspectorAuditProfile::AnalyzeFull,
    ]
    .into_iter()
    .map(|profile| audit_pdf_profile(path, file_size_bytes, profile))
    .collect()
}

#[must_use]
pub fn routing_decision(signals: &PdfInspectorRoutingSignals) -> PdfInspectorRoutingDecision {
    if signals.confidence < FAST_RUST_CONFIDENCE_THRESHOLD || signals.has_encoding_issues {
        return PdfInspectorRoutingDecision::FullDoclingFallback;
    }
    if matches!(signals.pdf_type, PdfInspectorPdfType::TextBased)
        && signals.pages_needing_ocr.is_empty()
        && !signals.is_complex
    {
        return PdfInspectorRoutingDecision::FastRustCandidate;
    }
    if !signals.pages_needing_ocr.is_empty() {
        return PdfInspectorRoutingDecision::HybridPageOcrCandidate;
    }
    PdfInspectorRoutingDecision::FullDoclingFallback
}

fn audit_pdf_profile(
    path: &Path,
    file_size_bytes: Option<u64>,
    profile: PdfInspectorAuditProfile,
) -> PdfInspectorAuditRecord {
    let Some(process_mode) = profile.process_mode() else {
        return unsupported_record(path, file_size_bytes);
    };
    let started = Instant::now();
    let detection = DetectionConfig {
        strategy: ScanStrategy::Full,
        ..DetectionConfig::default()
    };
    let options = PdfOptions::new().mode(process_mode).detection(detection);
    match process_pdf_with_options(path, options) {
        Ok(result) => success_record(
            path,
            file_size_bytes,
            profile,
            started.elapsed().as_secs_f64() * 1000.0,
            result,
        ),
        Err(error) => failed_record(
            path,
            file_size_bytes,
            profile,
            started.elapsed().as_secs_f64() * 1000.0,
            error.to_string(),
        ),
    }
}

fn success_record(
    path: &Path,
    file_size_bytes: Option<u64>,
    profile: PdfInspectorAuditProfile,
    elapsed_ms: f64,
    result: PdfProcessResult,
) -> PdfInspectorAuditRecord {
    let pdf_type = normalize_pdf_type(result.pdf_type);
    let signals = PdfInspectorRoutingSignals {
        pdf_type,
        page_count: result.page_count,
        confidence: result.confidence,
        pages_needing_ocr: result.pages_needing_ocr.clone(),
        is_complex: result.layout.is_complex,
        has_encoding_issues: result.has_encoding_issues,
    };
    PdfInspectorAuditRecord {
        source_path: path.to_string_lossy().to_string(),
        file_size_bytes,
        profile: profile.as_str().to_string(),
        pdf_type: Some(pdf_type.as_str().to_string()),
        page_count: Some(result.page_count),
        confidence: Some(result.confidence),
        pages_needing_ocr: result.pages_needing_ocr,
        is_complex: Some(result.layout.is_complex),
        pages_with_tables: result.layout.pages_with_tables,
        pages_with_columns: result.layout.pages_with_columns,
        has_encoding_issues: Some(result.has_encoding_issues),
        elapsed_ms,
        routing_decision: routing_decision(&signals).as_str().to_string(),
        error_message: None,
    }
}

fn failed_record(
    path: &Path,
    file_size_bytes: Option<u64>,
    profile: PdfInspectorAuditProfile,
    elapsed_ms: f64,
    error_message: String,
) -> PdfInspectorAuditRecord {
    PdfInspectorAuditRecord {
        source_path: path.to_string_lossy().to_string(),
        file_size_bytes,
        profile: profile.as_str().to_string(),
        pdf_type: None,
        page_count: None,
        confidence: None,
        pages_needing_ocr: Vec::new(),
        is_complex: None,
        pages_with_tables: Vec::new(),
        pages_with_columns: Vec::new(),
        has_encoding_issues: None,
        elapsed_ms,
        routing_decision: PdfInspectorRoutingDecision::PreflightFailed
            .as_str()
            .to_string(),
        error_message: Some(error_message),
    }
}

fn unsupported_record(path: &Path, file_size_bytes: Option<u64>) -> PdfInspectorAuditRecord {
    PdfInspectorAuditRecord {
        source_path: path.to_string_lossy().to_string(),
        file_size_bytes,
        profile: PdfInspectorAuditProfile::Unsupported.as_str().to_string(),
        pdf_type: None,
        page_count: None,
        confidence: None,
        pages_needing_ocr: Vec::new(),
        is_complex: None,
        pages_with_tables: Vec::new(),
        pages_with_columns: Vec::new(),
        has_encoding_issues: None,
        elapsed_ms: 0.0,
        routing_decision: PdfInspectorRoutingDecision::UnsupportedNonPdf
            .as_str()
            .to_string(),
        error_message: None,
    }
}

fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}

fn normalize_pdf_type(pdf_type: PdfType) -> PdfInspectorPdfType {
    match pdf_type {
        PdfType::TextBased => PdfInspectorPdfType::TextBased,
        PdfType::Scanned => PdfInspectorPdfType::Scanned,
        PdfType::ImageBased => PdfInspectorPdfType::ImageBased,
        PdfType::Mixed => PdfInspectorPdfType::Mixed,
    }
}

impl PdfInspectorPdfType {
    fn as_str(self) -> &'static str {
        match self {
            Self::TextBased => "text_based",
            Self::Scanned => "scanned",
            Self::ImageBased => "image_based",
            Self::Mixed => "mixed",
        }
    }
}

#[allow(dead_code)]
fn _assert_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LayoutComplexity>();
}

pub fn read_audit_paths_from_json(inputs_json: &str) -> Result<Vec<PathBuf>, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AuditInput {
        source: String,
    }

    let inputs = serde_json::from_str::<Vec<AuditInput>>(inputs_json)
        .map_err(|error| format!("decode PDF inspector audit inputs: {error}"))?;
    if inputs.is_empty() {
        return Err("PDF inspector audit inputs must not be empty".to_string());
    }
    Ok(inputs
        .into_iter()
        .map(|input| PathBuf::from(input.source))
        .collect())
}

pub fn write_audit_reports(
    report_dir: &Path,
    records: &[PdfInspectorAuditRecord],
) -> Result<(), String> {
    fs::create_dir_all(report_dir).map_err(|error| {
        format!(
            "create PDF inspector audit report directory `{}`: {error}",
            report_dir.display()
        )
    })?;
    let payload = serde_json::json!({
        "schema": "xiuxian_wendao.pdf_inspector_detect_audit.v1",
        "records": records,
        "summary": summarize_records(records),
    });
    let json_path = report_dir.join("pdf_inspector_detect_audit.json");
    fs::write(
        json_path.as_path(),
        serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("write PDF inspector audit JSON report: {error}"))?;
    let markdown_path = report_dir.join("pdf_inspector_detect_audit.md");
    fs::write(markdown_path.as_path(), render_markdown(records))
        .map_err(|error| format!("write PDF inspector audit Markdown report: {error}"))?;
    Ok(())
}

fn summarize_records(records: &[PdfInspectorAuditRecord]) -> serde_json::Value {
    let mut routing_counts = std::collections::BTreeMap::<String, usize>::new();
    let mut profile_counts = std::collections::BTreeMap::<String, usize>::new();
    for record in records {
        *routing_counts
            .entry(record.routing_decision.clone())
            .or_default() += 1;
        *profile_counts.entry(record.profile.clone()).or_default() += 1;
    }
    serde_json::json!({
        "recordCount": records.len(),
        "routingCounts": routing_counts,
        "profileCounts": profile_counts,
        "totalElapsedMs": records.iter().map(|record| record.elapsed_ms).sum::<f64>(),
    })
}

fn render_markdown(records: &[PdfInspectorAuditRecord]) -> String {
    let mut output = String::from("# Wendao PDF Inspector Detect Audit\n\n");
    output.push_str(
        "| Source | Profile | PDF type | Pages | Confidence | OCR pages | Complex | Encoding issues | Decision | Elapsed ms |\n",
    );
    output.push_str("| --- | --- | --- | ---: | ---: | --- | --- | --- | --- | ---: |\n");
    for record in records {
        let _ = writeln!(
            &mut output,
            "| `{}` | `{}` | `{}` | {} | {} | `{}` | {} | {} | `{}` | {:.3} |",
            record.source_path,
            record.profile,
            record.pdf_type.as_deref().unwrap_or(""),
            record
                .page_count
                .map_or_else(String::new, |value| value.to_string()),
            record
                .confidence
                .map_or_else(String::new, |value| format!("{value:.3}")),
            record
                .pages_needing_ocr
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(","),
            record
                .is_complex
                .map_or_else(String::new, |value| value.to_string()),
            record
                .has_encoding_issues
                .map_or_else(String::new, |value| value.to_string()),
            record.routing_decision,
            record.elapsed_ms,
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signals(pdf_type: PdfInspectorPdfType) -> PdfInspectorRoutingSignals {
        PdfInspectorRoutingSignals {
            pdf_type,
            page_count: 4,
            confidence: 0.95,
            pages_needing_ocr: Vec::new(),
            is_complex: false,
            has_encoding_issues: false,
        }
    }

    #[test]
    fn document_extract_pdf_audit_routes_simple_text_pdf_to_fast_candidate() {
        assert_eq!(
            routing_decision(&signals(PdfInspectorPdfType::TextBased)),
            PdfInspectorRoutingDecision::FastRustCandidate
        );
    }

    #[test]
    fn document_extract_pdf_audit_routes_mixed_pdf_to_hybrid_candidate() {
        let mut input = signals(PdfInspectorPdfType::Mixed);
        input.pages_needing_ocr = vec![2];

        assert_eq!(
            routing_decision(&input),
            PdfInspectorRoutingDecision::HybridPageOcrCandidate
        );
    }

    #[test]
    fn document_extract_pdf_audit_routes_scanned_and_image_pdf_to_hybrid_candidate() {
        for pdf_type in [
            PdfInspectorPdfType::Scanned,
            PdfInspectorPdfType::ImageBased,
        ] {
            let mut input = signals(pdf_type);
            input.pages_needing_ocr = vec![1, 2, 3, 4];

            assert_eq!(
                routing_decision(&input),
                PdfInspectorRoutingDecision::HybridPageOcrCandidate
            );
        }
    }

    #[test]
    fn document_extract_pdf_audit_routes_low_confidence_to_docling_fallback() {
        let mut input = signals(PdfInspectorPdfType::TextBased);
        input.confidence = 0.5;

        assert_eq!(
            routing_decision(&input),
            PdfInspectorRoutingDecision::FullDoclingFallback
        );
    }

    #[test]
    fn document_extract_pdf_audit_routes_encoding_issues_to_docling_fallback() {
        let mut input = signals(PdfInspectorPdfType::TextBased);
        input.has_encoding_issues = true;

        assert_eq!(
            routing_decision(&input),
            PdfInspectorRoutingDecision::FullDoclingFallback
        );
    }

    #[test]
    fn document_extract_pdf_audit_routes_complex_text_pdf_to_docling_fallback() {
        let mut input = signals(PdfInspectorPdfType::TextBased);
        input.is_complex = true;

        assert_eq!(
            routing_decision(&input),
            PdfInspectorRoutingDecision::FullDoclingFallback
        );
    }

    #[test]
    fn document_extract_pdf_audit_marks_non_pdf_as_unsupported() {
        let records = audit_pdf_path(Path::new("sample.docx"));

        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].routing_decision,
            PdfInspectorRoutingDecision::UnsupportedNonPdf.as_str()
        );
    }

    #[test]
    fn document_extract_pdf_audit_marks_invalid_pdf_as_preflight_failed() -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("broken.pdf");
        fs::write(source.as_path(), b"not a pdf").map_err(|error| error.to_string())?;

        let records = audit_pdf_path(source.as_path());

        assert_eq!(records.len(), 2);
        assert!(records.iter().all(|record| record.routing_decision
            == PdfInspectorRoutingDecision::PreflightFailed.as_str()));
        Ok(())
    }
}
