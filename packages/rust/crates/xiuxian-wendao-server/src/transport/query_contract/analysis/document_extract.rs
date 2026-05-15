//! Document-extraction route contract and metadata validation.

/// Stable route for the document extraction analysis contract.
pub const ANALYSIS_DOCUMENT_EXTRACT_ROUTE: &str = "/analysis/document-extract";
/// Stable route for the Rust-owned document extraction job status contract.
pub const ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE: &str = "/analysis/document-extract-status";
/// Internal route for page-shard OCR exchange with the Python analyzer worker.
pub const ANALYSIS_PDF_OCR_SHARDS_ROUTE: &str = "/analysis/pdf-ocr-shards";
/// Internal route for audio-shard exchange with the Python analyzer worker.
pub const ANALYSIS_AUDIO_SHARDS_ROUTE: &str = "/analysis/audio-shards";

/// Canonical document source-path metadata header for Wendao Flight requests.
pub const WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: &str =
    "x-wendao-document-extract-source-path";
/// Canonical document output-dir metadata header for Wendao Flight requests.
pub const WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER: &str = "x-wendao-document-extract-output-dir";
/// Canonical document force-refresh metadata header for Wendao Flight requests.
pub const WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER: &str = "x-wendao-document-extract-force";
/// Canonical document error-row metadata header for Wendao Flight requests.
pub const WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER: &str = "x-wendao-document-extract-error-row";
/// Canonical document extraction profile header for Python analyzer workers.
pub const WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: &str = "x-wendao-document-extract-profile";
/// Internal 1-based inclusive page-range header for Python analyzer workers.
pub const WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER: &str = "x-wendao-document-extract-page-range";
/// Canonical document extraction mode header for Rust-owned queueing.
pub const WENDAO_DOCUMENT_EXTRACT_MODE_HEADER: &str = "x-wendao-document-extract-mode";
/// Canonical async wait budget header in milliseconds.
pub const WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER: &str = "x-wendao-document-extract-wait-ms";
/// Canonical document extraction job-id header for status requests.
pub const WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER: &str = "x-wendao-document-extract-job-id";
/// Internal PDF OCR worker budget header for Python shard OCR requests.
pub const WENDAO_PDF_OCR_WORKERS_HEADER: &str = "x-wendao-pdf-ocr-workers";
/// Internal audio worker budget header for Python audio shard requests.
pub const WENDAO_AUDIO_WORKERS_HEADER: &str = "x-wendao-audio-workers";

/// Full Docling document extraction profile.
pub const DOCUMENT_EXTRACT_FULL_PROFILE: &str = "full";
/// Lightweight text-first document extraction profile for chat attachments.
pub const DOCUMENT_EXTRACT_FAST_TEXT_PROFILE: &str = "fast-text";

/// Document extraction execution mode decoded from Flight metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentExtractMode {
    /// Run the conversion synchronously through the Python Arrow Flight worker.
    Sync,
    /// Queue first-time conversion in the Rust provider and return job state.
    Async,
    /// Explicit opt-in hybrid route that renders PDF page OCR shards in Rust.
    HybridPageOcr,
}

impl DocumentExtractMode {
    /// Decode a metadata value into a document extraction mode.
    ///
    /// # Errors
    ///
    /// Returns an error for values outside `sync`, `async`, and
    /// `hybrid-page-ocr`.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "sync" => Ok(Self::Sync),
            "async" => Ok(Self::Async),
            "hybrid-page-ocr" | "hybrid_page_ocr" => Ok(Self::HybridPageOcr),
            other => Err(format!("invalid document extract mode `{other}`")),
        }
    }
}

/// Transport-owned document extraction request decoded from Flight metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentExtractFlightRequest {
    /// Source document path.
    pub source_path: String,
    /// Output directory for extracted resources.
    pub output_dir: String,
    /// Force reconversion even when cache artifacts exist.
    pub force: bool,
    /// Return table-shaped error rows when the worker fails.
    pub error_row: bool,
    /// Python analyzer extraction profile.
    pub profile: String,
    /// Execution mode for the Rust provider.
    pub mode: DocumentExtractMode,
    /// Async wait budget in milliseconds.
    pub wait_ms: u64,
}

/// Validate the stable document extraction request contract.
///
/// # Errors
///
/// Returns an error when the source path is blank.
pub fn validate_document_extract_request(source_path: &str) -> Result<(), String> {
    if source_path.trim().is_empty() {
        return Err("document extract source path must not be blank".to_string());
    }
    Ok(())
}

/// Normalize a document extraction profile for transport metadata.
///
/// # Errors
///
/// Returns an error when the profile is not recognized.
pub fn normalize_document_extract_profile(profile: &str) -> Result<&'static str, String> {
    match profile.trim().to_ascii_lowercase().as_str() {
        "" | "default" | "docling" | "full" | "full-docling" => Ok(DOCUMENT_EXTRACT_FULL_PROFILE),
        "attachment" | "attachment-fast-text" | "fast" | "fast_text" | "fast-text" | "text" => {
            Ok(DOCUMENT_EXTRACT_FAST_TEXT_PROFILE)
        }
        other => Err(format!("unsupported document extract profile `{other}`")),
    }
}
