/// Stable route for the PDF extraction analysis contract.
pub const ANALYSIS_PDF_EXTRACT_ROUTE: &str = "/analysis/pdf-extract";

/// Canonical PDF source-path metadata header for Wendao Flight requests.
pub const WENDAO_PDF_EXTRACT_SOURCE_PATH_HEADER: &str = "x-wendao-pdf-extract-source-path";
/// Canonical PDF output-dir metadata header for Wendao Flight requests.
pub const WENDAO_PDF_EXTRACT_OUTPUT_DIR_HEADER: &str = "x-wendao-pdf-extract-output-dir";
/// Canonical PDF extract-images flag header for Wendao Flight requests.
pub const WENDAO_PDF_EXTRACT_IMAGES_HEADER: &str = "x-wendao-pdf-extract-images";
/// Canonical PDF extract-tables flag header for Wendao Flight requests.
pub const WENDAO_PDF_EXTRACT_TABLES_HEADER: &str = "x-wendao-pdf-extract-tables";
/// Canonical PDF extract-formulas flag header for Wendao Flight requests.
pub const WENDAO_PDF_EXTRACT_FORMULAS_HEADER: &str = "x-wendao-pdf-extract-formulas";

/// Validate the stable PDF extraction request contract.
///
/// # Errors
///
/// Returns an error when the source path is blank.
pub fn validate_pdf_extract_request(source_path: &str) -> Result<(), String> {
    if source_path.trim().is_empty() {
        return Err("PDF extract source path must not be blank".to_string());
    }
    Ok(())
}
