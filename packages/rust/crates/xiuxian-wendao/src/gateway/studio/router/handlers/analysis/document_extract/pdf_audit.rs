use std::fmt::Write as _;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use arrow::array::{ArrayRef, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use pdf_inspector::{
    DetectionConfig, LayoutComplexity, PagesExtractionResult, PdfOptions, PdfProcessResult,
    PdfType, ProcessMode, ScanStrategy, extract_pages_markdown, process_pdf_with_options,
};
use serde::{Deserialize, Serialize};

const FAST_RUST_CONFIDENCE_THRESHOLD: f32 = 0.90;
const DOCUMENT_RESOURCE_ARROW_CACHE_NAME: &str = "_resources.arrow";
const DOCUMENT_EXTRACT_COMPLETE_MARKER_NAME: &str = "_complete.marker";
const PDF_INSPECTOR_TEXT_FAST_PATH_PROFILE: &str = "pdf-inspector-text-fast-path-v1";
const PDF_INSPECTOR_DEPENDENCY_PIN: &str =
    "tao3k/pdf-inspector@xiuxian#63b55731337c18baf23319b73cc9780bb23ac61b";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfInspectorTextFastPathConfig {
    pub enabled: bool,
}

impl PdfInspectorTextFastPathConfig {
    #[must_use]
    pub fn enabled() -> Self {
        Self { enabled: true }
    }

    #[must_use]
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}

impl Default for PdfInspectorTextFastPathConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfInspectorTextFastPathRecord {
    pub source_path: String,
    pub output_dir: String,
    pub artifact_path: Option<String>,
    pub arrow_cache_path: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub inspector_version: String,
    pub routing_profile: String,
    pub converter_profile: String,
    pub pdf_type: Option<String>,
    pub page_count: Option<u32>,
    pub pages_extracted: u32,
    pub pages_needing_ocr: Vec<u32>,
    pub is_complex: Option<bool>,
    pub has_encoding_issues: Option<bool>,
    pub confidence: Option<f32>,
    pub markdown_bytes: u64,
    pub arrow_rows: u32,
    pub elapsed_ms: f64,
    pub routing_decision: String,
    pub status: String,
    pub error_message: Option<String>,
}

struct TextFastPathContext<'a> {
    path: &'a Path,
    output_dir: &'a Path,
    file_size_bytes: Option<u64>,
    started: Instant,
}

impl<'a> TextFastPathContext<'a> {
    fn new(path: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            path,
            output_dir,
            file_size_bytes: path.metadata().ok().map(|metadata| metadata.len()),
            started: Instant::now(),
        }
    }

    fn elapsed_ms(&self) -> f64 {
        self.started.elapsed().as_secs_f64() * 1000.0
    }

    fn record(&self, parts: TextFastPathRecordParts) -> PdfInspectorTextFastPathRecord {
        PdfInspectorTextFastPathRecord {
            source_path: self.path.to_string_lossy().to_string(),
            output_dir: self.output_dir.to_string_lossy().to_string(),
            artifact_path: parts
                .artifact_path
                .map(|path| path.to_string_lossy().to_string()),
            arrow_cache_path: parts
                .arrow_cache_path
                .map(|path| path.to_string_lossy().to_string()),
            file_size_bytes: self.file_size_bytes,
            inspector_version: PDF_INSPECTOR_DEPENDENCY_PIN.to_string(),
            routing_profile: PDF_INSPECTOR_TEXT_FAST_PATH_PROFILE.to_string(),
            converter_profile: text_fast_path_converter_profile(),
            pdf_type: parts.pdf_type,
            page_count: parts.page_count,
            pages_extracted: parts.page_count.unwrap_or_default(),
            pages_needing_ocr: parts.pages_needing_ocr,
            is_complex: parts.is_complex,
            has_encoding_issues: parts.has_encoding_issues,
            confidence: parts.confidence,
            markdown_bytes: parts.markdown_bytes,
            arrow_rows: parts.arrow_rows,
            elapsed_ms: self.elapsed_ms(),
            routing_decision: parts.routing_decision.as_str().to_string(),
            status: parts.status,
            error_message: parts.error_message,
        }
    }
}

struct TextFastPathRecordParts {
    pdf_type: Option<String>,
    page_count: Option<u32>,
    pages_needing_ocr: Vec<u32>,
    is_complex: Option<bool>,
    has_encoding_issues: Option<bool>,
    confidence: Option<f32>,
    markdown_bytes: u64,
    arrow_rows: u32,
    routing_decision: PdfInspectorRoutingDecision,
    status: String,
    error_message: Option<String>,
    artifact_path: Option<PathBuf>,
    arrow_cache_path: Option<PathBuf>,
}

impl TextFastPathRecordParts {
    fn new(routing_decision: PdfInspectorRoutingDecision, status: &str) -> Self {
        Self {
            pdf_type: None,
            page_count: None,
            pages_needing_ocr: Vec::new(),
            is_complex: None,
            has_encoding_issues: None,
            confidence: None,
            markdown_bytes: 0,
            arrow_rows: 0,
            routing_decision,
            status: status.to_string(),
            error_message: None,
            artifact_path: None,
            arrow_cache_path: None,
        }
    }

    fn with_error(mut self, error_message: String) -> Self {
        self.error_message = Some(error_message);
        self
    }
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
pub fn text_fast_path_converter_profile() -> String {
    format!("{PDF_INSPECTOR_DEPENDENCY_PIN}:{PDF_INSPECTOR_TEXT_FAST_PATH_PROFILE}")
}

#[must_use]
pub fn audit_pdf_paths(paths: &[PathBuf]) -> Vec<PdfInspectorAuditRecord> {
    paths
        .iter()
        .flat_map(|path| audit_pdf_path(path.as_path()))
        .collect()
}

#[must_use]
pub fn extract_text_pdf_fast_path_artifacts(
    paths: &[PathBuf],
    artifact_root: &Path,
    config: &PdfInspectorTextFastPathConfig,
) -> Vec<PdfInspectorTextFastPathRecord> {
    paths
        .iter()
        .map(|path| {
            let output_dir = artifact_root.join(output_dir_name_for_source(path.as_path()));
            extract_text_pdf_fast_path_artifact(path.as_path(), output_dir.as_path(), config)
        })
        .collect()
}

#[must_use]
pub fn extract_text_pdf_fast_path_artifact(
    path: &Path,
    output_dir: &Path,
    config: &PdfInspectorTextFastPathConfig,
) -> PdfInspectorTextFastPathRecord {
    let context = TextFastPathContext::new(path, output_dir);
    if !config.enabled {
        return context.record(TextFastPathRecordParts::new(
            PdfInspectorRoutingDecision::FullDoclingFallback,
            "disabled",
        ));
    }
    if !is_pdf_path(path) {
        return context.record(TextFastPathRecordParts::new(
            PdfInspectorRoutingDecision::UnsupportedNonPdf,
            "unsupported",
        ));
    }

    let analysis = match analyze_pdf_for_text_fast_path(path) {
        Ok(result) => result,
        Err(error) => {
            return context.record(
                TextFastPathRecordParts::new(PdfInspectorRoutingDecision::PreflightFailed, "error")
                    .with_error(error),
            );
        }
    };

    let pdf_type = normalize_pdf_type(analysis.pdf_type);
    let decision = routing_decision(&signals_from_analysis(&analysis, pdf_type));
    if decision != PdfInspectorRoutingDecision::FastRustCandidate {
        return context.record(parts_from_analysis(
            &analysis, pdf_type, decision, "fallback",
        ));
    }

    let pages = match extract_pages_markdown(path, None) {
        Ok(pages) => pages,
        Err(error) => {
            return context.record(
                parts_from_analysis(
                    &analysis,
                    pdf_type,
                    PdfInspectorRoutingDecision::FullDoclingFallback,
                    "error",
                )
                .with_error(error.to_string()),
            );
        }
    };
    let pages_needing_ocr = pages_needing_ocr_from_extraction(&pages);
    if pages.is_complex || !pages_needing_ocr.is_empty() {
        return context.record(parts_from_pages(
            &analysis,
            pdf_type,
            &pages,
            pages_needing_ocr,
            PdfInspectorRoutingDecision::FullDoclingFallback,
            "fallback",
        ));
    }

    let markdown = markdown_from_pages(&pages);
    if markdown.trim().is_empty() {
        return context.record(
            parts_from_pages(
                &analysis,
                pdf_type,
                &pages,
                Vec::new(),
                PdfInspectorRoutingDecision::FullDoclingFallback,
                "fallback",
            )
            .with_error("pdf-inspector produced empty markdown".to_string()),
        );
    }

    match write_text_fast_path_artifact(path, output_dir, markdown.as_str()) {
        Ok((artifact_path, arrow_cache_path, batch)) => context.record(ok_parts_from_artifact(
            &analysis,
            pdf_type,
            &pages,
            &markdown,
            batch.num_rows(),
            artifact_path,
            arrow_cache_path,
        )),
        Err(error) => {
            let mut parts = parts_from_pages(
                &analysis,
                pdf_type,
                &pages,
                Vec::new(),
                PdfInspectorRoutingDecision::FullDoclingFallback,
                "error",
            )
            .with_error(error);
            parts.markdown_bytes = markdown.len() as u64;
            context.record(parts)
        }
    }
}

fn analyze_pdf_for_text_fast_path(path: &Path) -> Result<PdfProcessResult, String> {
    let detection = DetectionConfig {
        strategy: ScanStrategy::Full,
        ..DetectionConfig::default()
    };
    let options = PdfOptions::new()
        .mode(ProcessMode::Analyze)
        .detection(detection);
    process_pdf_with_options(path, options).map_err(|error| error.to_string())
}

fn signals_from_analysis(
    analysis: &PdfProcessResult,
    pdf_type: PdfInspectorPdfType,
) -> PdfInspectorRoutingSignals {
    PdfInspectorRoutingSignals {
        pdf_type,
        page_count: analysis.page_count,
        confidence: analysis.confidence,
        pages_needing_ocr: analysis.pages_needing_ocr.clone(),
        is_complex: analysis.layout.is_complex,
        has_encoding_issues: analysis.has_encoding_issues,
    }
}

fn parts_from_analysis(
    analysis: &PdfProcessResult,
    pdf_type: PdfInspectorPdfType,
    decision: PdfInspectorRoutingDecision,
    status: &str,
) -> TextFastPathRecordParts {
    let mut parts = TextFastPathRecordParts::new(decision, status);
    parts.pdf_type = Some(pdf_type.as_str().to_string());
    parts.page_count = Some(analysis.page_count);
    parts
        .pages_needing_ocr
        .clone_from(&analysis.pages_needing_ocr);
    parts.is_complex = Some(analysis.layout.is_complex);
    parts.has_encoding_issues = Some(analysis.has_encoding_issues);
    parts.confidence = Some(analysis.confidence);
    parts
}

fn parts_from_pages(
    analysis: &PdfProcessResult,
    pdf_type: PdfInspectorPdfType,
    pages: &PagesExtractionResult,
    pages_needing_ocr: Vec<u32>,
    decision: PdfInspectorRoutingDecision,
    status: &str,
) -> TextFastPathRecordParts {
    let mut parts = parts_from_analysis(analysis, pdf_type, decision, status);
    parts.pages_needing_ocr = pages_needing_ocr;
    parts.is_complex = Some(pages.is_complex);
    parts
}

fn ok_parts_from_artifact(
    analysis: &PdfProcessResult,
    pdf_type: PdfInspectorPdfType,
    pages: &PagesExtractionResult,
    markdown: &str,
    row_count: usize,
    artifact_path: PathBuf,
    arrow_cache_path: PathBuf,
) -> TextFastPathRecordParts {
    let mut parts = parts_from_pages(
        analysis,
        pdf_type,
        pages,
        Vec::new(),
        PdfInspectorRoutingDecision::FastRustCandidate,
        "ok",
    );
    parts.markdown_bytes = markdown.len() as u64;
    parts.arrow_rows = u32::try_from(row_count).unwrap_or(u32::MAX);
    parts.artifact_path = Some(artifact_path);
    parts.arrow_cache_path = Some(arrow_cache_path);
    parts
}

fn pages_needing_ocr_from_extraction(pages: &PagesExtractionResult) -> Vec<u32> {
    let mut pages_needing_ocr = pages.pages_needing_ocr.clone();
    pages_needing_ocr.extend(
        pages
            .pages
            .iter()
            .filter(|page| page.needs_ocr)
            .map(|page| page.page + 1),
    );
    pages_needing_ocr.sort_unstable();
    pages_needing_ocr.dedup();
    pages_needing_ocr
}

fn markdown_from_pages(pages: &PagesExtractionResult) -> String {
    pages
        .pages
        .iter()
        .map(|page| page.markdown.trim())
        .filter(|markdown| !markdown.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
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

fn write_text_fast_path_artifact(
    source_path: &Path,
    output_dir: &Path,
    markdown: &str,
) -> Result<(PathBuf, PathBuf, RecordBatch), String> {
    fs::create_dir_all(output_dir).map_err(|error| {
        format!(
            "create pdf-inspector text fast-path artifact directory `{}`: {error}",
            output_dir.display()
        )
    })?;
    let markdown_path = output_dir.join(format!(
        "{}.md",
        source_path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .filter(|stem| !stem.trim().is_empty())
            .unwrap_or("document")
    ));
    fs::write(markdown_path.as_path(), markdown).map_err(|error| {
        format!(
            "write pdf-inspector text fast-path markdown `{}`: {error}",
            markdown_path.display()
        )
    })?;
    let batch =
        build_text_fast_path_resource_batch(source_path, markdown_path.as_path(), markdown)?;
    let arrow_cache_path = output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
    write_arrow_file(arrow_cache_path.as_path(), std::slice::from_ref(&batch))?;
    File::create(output_dir.join(DOCUMENT_EXTRACT_COMPLETE_MARKER_NAME))
        .map_err(|error| format!("touch pdf-inspector text fast-path marker: {error}"))?;
    Ok((markdown_path, arrow_cache_path, batch))
}

fn build_text_fast_path_resource_batch(
    source_path: &Path,
    markdown_path: &Path,
    markdown: &str,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            string_column([source_path.to_string_lossy().as_ref()]),
            string_column(["document"]),
            string_column([markdown_path.to_string_lossy().as_ref()]),
            Arc::new(Int32Array::from(vec![0])) as ArrayRef,
            string_column([""]),
            string_column([markdown]),
            string_column(["text/markdown"]),
            string_column(["ok"]),
            string_column(["_main"]),
        ],
    )
    .map_err(|error| format!("build pdf-inspector text fast-path resource batch: {error}"))
}

fn document_resource_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("resourceType", DataType::Utf8, true),
        Field::new("resourcePath", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("caption", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("elementId", DataType::Utf8, true),
    ]))
}

fn write_arrow_file(path: &Path, batches: &[RecordBatch]) -> Result<(), String> {
    let Some(first) = batches.first() else {
        return Err(format!(
            "cannot write empty Arrow IPC file `{}`",
            path.display()
        ));
    };
    let file = File::create(path)
        .map_err(|error| format!("create Arrow IPC file `{}`: {error}", path.display()))?;
    let mut writer = FileWriter::try_new(file, first.schema().as_ref())
        .map_err(|error| format!("create Arrow IPC writer `{}`: {error}", path.display()))?;
    for batch in batches {
        writer
            .write(batch)
            .map_err(|error| format!("write Arrow IPC batch `{}`: {error}", path.display()))?;
    }
    writer
        .finish()
        .map_err(|error| format!("finish Arrow IPC file `{}`: {error}", path.display()))
}

fn string_column<'a>(values: impl IntoIterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as ArrayRef
}

fn output_dir_name_for_source(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("document.pdf");
    format!("{name}.pdf-inspector-text-fast-path")
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

pub fn write_text_fast_path_reports(
    report_dir: &Path,
    records: &[PdfInspectorTextFastPathRecord],
) -> Result<(), String> {
    fs::create_dir_all(report_dir).map_err(|error| {
        format!(
            "create PDF inspector text fast-path report directory `{}`: {error}",
            report_dir.display()
        )
    })?;
    let payload = serde_json::json!({
        "schema": "xiuxian_wendao.pdf_inspector_text_fast_path.v1",
        "records": records,
        "summary": summarize_text_fast_path_records(records),
    });
    let json_path = report_dir.join("pdf_inspector_text_fast_path.json");
    fs::write(
        json_path.as_path(),
        serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("write PDF inspector text fast-path JSON report: {error}"))?;
    let markdown_path = report_dir.join("pdf_inspector_text_fast_path.md");
    fs::write(
        markdown_path.as_path(),
        render_text_fast_path_markdown(records),
    )
    .map_err(|error| format!("write PDF inspector text fast-path Markdown report: {error}"))?;
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

fn summarize_text_fast_path_records(
    records: &[PdfInspectorTextFastPathRecord],
) -> serde_json::Value {
    let mut status_counts = std::collections::BTreeMap::<String, usize>::new();
    let mut routing_counts = std::collections::BTreeMap::<String, usize>::new();
    for record in records {
        *status_counts.entry(record.status.clone()).or_default() += 1;
        *routing_counts
            .entry(record.routing_decision.clone())
            .or_default() += 1;
    }
    serde_json::json!({
        "recordCount": records.len(),
        "statusCounts": status_counts,
        "routingCounts": routing_counts,
        "totalElapsedMs": records.iter().map(|record| record.elapsed_ms).sum::<f64>(),
        "totalMarkdownBytes": records.iter().map(|record| record.markdown_bytes).sum::<u64>(),
        "totalArrowRows": records.iter().map(|record| record.arrow_rows).sum::<u32>(),
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

fn render_text_fast_path_markdown(records: &[PdfInspectorTextFastPathRecord]) -> String {
    let mut output = String::from("# Wendao PDF Inspector Text Fast Path\n\n");
    output.push_str(
        "| Source | Status | PDF type | Pages | OCR pages | Complex | Decision | Markdown bytes | Arrow rows | Elapsed ms |\n",
    );
    output.push_str("| --- | --- | --- | ---: | --- | --- | --- | ---: | ---: | ---: |\n");
    for record in records {
        let _ = writeln!(
            &mut output,
            "| `{}` | `{}` | `{}` | {} | `{}` | {} | `{}` | {} | {} | {:.3} |",
            record.source_path,
            record.status,
            record.pdf_type.as_deref().unwrap_or(""),
            record
                .page_count
                .map_or_else(String::new, |value| value.to_string()),
            record
                .pages_needing_ocr
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(","),
            record
                .is_complex
                .map_or_else(String::new, |value| value.to_string()),
            record.routing_decision,
            record.markdown_bytes,
            record.arrow_rows,
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

    fn minimal_text_pdf_add_object(
        pdf: &mut Vec<u8>,
        offsets: &mut Vec<usize>,
        id: usize,
        body: &str,
    ) {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        pdf.extend_from_slice(body.as_bytes());
        pdf.extend_from_slice(b"\nendobj\n");
    }

    fn minimal_text_pdf() -> Vec<u8> {
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = vec![0_usize];

        minimal_text_pdf_add_object(
            &mut pdf,
            &mut offsets,
            1,
            "<< /Type /Catalog /Pages 2 0 R >>",
        );
        minimal_text_pdf_add_object(
            &mut pdf,
            &mut offsets,
            2,
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        );
        minimal_text_pdf_add_object(
            &mut pdf,
            &mut offsets,
            3,
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
        );
        let content = "BT /F1 12 Tf 100 700 Td (Hello World) Tj 0 -14 Td (Second Line) Tj 0 -14 Td (Third Line) Tj ET";
        minimal_text_pdf_add_object(
            &mut pdf,
            &mut offsets,
            4,
            &format!(
                "<< /Length {} >>\nstream\n{}\nendstream",
                content.len(),
                content
            ),
        );
        minimal_text_pdf_add_object(
            &mut pdf,
            &mut offsets,
            5,
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        );

        let xref_start = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", offsets.len()).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets.iter().skip(1) {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
                offsets.len(),
                xref_start
            )
            .as_bytes(),
        );

        pdf
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

    #[test]
    fn document_extract_pdf_text_fast_path_can_be_disabled_by_config() {
        let record = extract_text_pdf_fast_path_artifact(
            Path::new("sample.pdf"),
            Path::new("sample.pdf.extracted"),
            &PdfInspectorTextFastPathConfig::disabled(),
        );

        assert_eq!(record.status, "disabled");
        assert_eq!(
            record.converter_profile,
            "tao3k/pdf-inspector@xiuxian#63b55731337c18baf23319b73cc9780bb23ac61b:pdf-inspector-text-fast-path-v1"
        );
        assert_eq!(record.arrow_rows, 0);
    }

    #[test]
    fn document_extract_pdf_text_fast_path_writes_stable_arrow_resource_row() -> Result<(), String>
    {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("sample.pdf");
        let output_dir = temp.path().join("sample.pdf.extracted");
        fs::write(source.as_path(), b"%PDF-1.4\n").map_err(|error| error.to_string())?;

        let (markdown_path, arrow_cache_path, batch) =
            write_text_fast_path_artifact(source.as_path(), output_dir.as_path(), "# Sample\n")?;

        assert_eq!(markdown_path, output_dir.join("sample.md"));
        assert_eq!(
            arrow_cache_path,
            output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
        );
        assert!(
            output_dir
                .join(DOCUMENT_EXTRACT_COMPLETE_MARKER_NAME)
                .exists()
        );
        assert_eq!(batch.num_rows(), 1);
        let schema = batch.schema();
        let field_names = schema
            .fields()
            .iter()
            .map(|field| field.name().as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            field_names,
            vec![
                "sourcePath",
                "resourceType",
                "resourcePath",
                "pageIndex",
                "caption",
                "content",
                "mimeType",
                "status",
                "elementId",
            ]
        );
        Ok(())
    }

    #[test]
    fn document_extract_pdf_text_fast_path_extracts_simple_text_pdf() -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("simple.pdf");
        let output_dir = temp.path().join("simple.pdf.extracted");
        fs::write(source.as_path(), minimal_text_pdf()).map_err(|error| error.to_string())?;

        let record = extract_text_pdf_fast_path_artifact(
            source.as_path(),
            output_dir.as_path(),
            &PdfInspectorTextFastPathConfig::enabled(),
        );

        assert_eq!(record.status, "ok", "{record:?}");
        assert_eq!(record.routing_decision, "fast_rust_candidate");
        assert_eq!(record.arrow_rows, 1);
        assert!(record.markdown_bytes > 0);
        let markdown_path = record
            .artifact_path
            .as_deref()
            .ok_or_else(|| "missing markdown artifact path".to_string())?;
        let markdown =
            fs::read_to_string(markdown_path).map_err(|error| format!("read markdown: {error}"))?;
        assert!(markdown.contains("Hello World"));
        assert!(markdown.contains("Second Line"));
        assert!(record.arrow_cache_path.is_some());
        Ok(())
    }
}
