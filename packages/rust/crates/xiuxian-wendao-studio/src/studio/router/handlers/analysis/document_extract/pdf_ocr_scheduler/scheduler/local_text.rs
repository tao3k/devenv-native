use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PdfOcrShardInput, PdfOcrShardResult,
};
use xiuxian_wendao_attachments::pdf::text::source_pdf_page_text_results;

pub(crate) const DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT";
pub(crate) const DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT";
pub(crate) const DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_EMPTY_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_EMPTY";
const LOCAL_TEXT_RUST_LOPDF_MODE: &str = "rust-lopdf";
const LOCAL_BACKEND_TEXT_EMPTY_DISPATCH_PYTHON_MODE: &str = "dispatch-python";
const LOCAL_BACKEND_TEXT_EMPTY_FAIL_FAST_MODE: &str = "fail-fast";
const SOURCE_PDF_PAGE_IMAGE_MIME_TYPE: &str = "application/x-wendao-source-pdf-page";

#[derive(Debug, Clone, Copy)]
struct LocalTextModes {
    backend_text: bool,
    fast_text: bool,
    backend_text_empty: LocalBackendTextEmptyMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalBackendTextEmptyMode {
    DispatchPython,
    FailFast,
}

impl LocalTextModes {
    fn from_env() -> Self {
        Self {
            backend_text: local_text_env_enabled(DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_ENV),
            fast_text: local_text_env_enabled(DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT_ENV),
            backend_text_empty: local_backend_text_empty_mode(),
        }
    }

    #[cfg(test)]
    fn backend_text_only() -> Self {
        Self {
            backend_text: true,
            fast_text: false,
            backend_text_empty: LocalBackendTextEmptyMode::DispatchPython,
        }
    }

    #[cfg(test)]
    fn backend_and_fast_text() -> Self {
        Self {
            backend_text: true,
            fast_text: true,
            backend_text_empty: LocalBackendTextEmptyMode::DispatchPython,
        }
    }

    #[cfg(test)]
    fn backend_text_empty_fail_fast() -> Self {
        Self {
            backend_text: true,
            fast_text: false,
            backend_text_empty: LocalBackendTextEmptyMode::FailFast,
        }
    }

    fn any_enabled(self) -> bool {
        self.backend_text || self.fast_text
    }
}

pub(super) fn local_backend_text_results(
    inputs: &[PdfOcrShardInput],
) -> Vec<Option<PdfOcrShardResult>> {
    let modes = LocalTextModes::from_env();
    if !modes.any_enabled() {
        return vec![None; inputs.len()];
    }
    local_text_results_enabled(inputs, modes)
}

#[cfg(test)]
pub(crate) fn local_backend_text_results_for_tests(
    inputs: &[PdfOcrShardInput],
) -> Vec<Option<PdfOcrShardResult>> {
    local_text_results_enabled(inputs, LocalTextModes::backend_text_only())
}

#[cfg(test)]
pub(crate) fn local_backend_and_fast_text_results_for_tests(
    inputs: &[PdfOcrShardInput],
) -> Vec<Option<PdfOcrShardResult>> {
    local_text_results_enabled(inputs, LocalTextModes::backend_and_fast_text())
}

#[cfg(test)]
pub(crate) fn local_empty_backend_text_dispatch_python_results_for_tests(
    inputs: &[PdfOcrShardInput],
) -> Vec<Option<PdfOcrShardResult>> {
    local_text_results_enabled_with_extractor(
        inputs,
        LocalTextModes::backend_text_only(),
        empty_text_extractor,
    )
}

#[cfg(test)]
pub(crate) fn local_empty_backend_text_fail_fast_results_for_tests(
    inputs: &[PdfOcrShardInput],
) -> Vec<Option<PdfOcrShardResult>> {
    local_text_results_enabled_with_extractor(
        inputs,
        LocalTextModes::backend_text_empty_fail_fast(),
        empty_text_extractor,
    )
}

#[cfg(test)]
pub(crate) fn local_backend_text_error_fail_fast_results_for_tests(
    inputs: &[PdfOcrShardInput],
) -> Vec<Option<PdfOcrShardResult>> {
    local_text_results_enabled_with_extractor(
        inputs,
        LocalTextModes::backend_text_empty_fail_fast(),
        error_text_extractor,
    )
}

#[cfg(test)]
pub(crate) fn local_partial_backend_text_error_fail_fast_results_for_tests(
    inputs: &[PdfOcrShardInput],
) -> Vec<Option<PdfOcrShardResult>> {
    local_text_results_enabled_with_extractor(
        inputs,
        LocalTextModes::backend_text_empty_fail_fast(),
        partial_error_text_extractor,
    )
}

fn local_text_env_enabled(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| {
            value.trim().replace('_', "-").to_ascii_lowercase() == LOCAL_TEXT_RUST_LOPDF_MODE
        })
        .unwrap_or(false)
}

fn local_backend_text_empty_mode() -> LocalBackendTextEmptyMode {
    local_backend_text_empty_mode_with_lookup(&|key| std::env::var(key))
}

fn local_backend_text_empty_mode_with_lookup(
    lookup: &dyn Fn(&str) -> Result<String, std::env::VarError>,
) -> LocalBackendTextEmptyMode {
    let value = lookup(DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_EMPTY_ENV)
        .ok()
        .unwrap_or_else(|| LOCAL_BACKEND_TEXT_EMPTY_DISPATCH_PYTHON_MODE.to_string());
    match value.trim().replace('_', "-").to_ascii_lowercase().as_str() {
        LOCAL_BACKEND_TEXT_EMPTY_FAIL_FAST_MODE => LocalBackendTextEmptyMode::FailFast,
        _ => LocalBackendTextEmptyMode::DispatchPython,
    }
}

fn local_text_results_enabled(
    inputs: &[PdfOcrShardInput],
    modes: LocalTextModes,
) -> Vec<Option<PdfOcrShardResult>> {
    local_text_results_enabled_with_extractor(inputs, modes, source_pdf_page_text_results)
}

fn local_text_results_enabled_with_extractor(
    inputs: &[PdfOcrShardInput],
    modes: LocalTextModes,
    extractor: impl Fn(&Path, &[u32]) -> Result<Vec<Result<String, String>>, String>,
) -> Vec<Option<PdfOcrShardResult>> {
    let mut results = vec![None; inputs.len()];
    let mut groups: BTreeMap<PathBuf, Vec<(usize, u32)>> = BTreeMap::new();

    for (position, input) in inputs.iter().enumerate() {
        if !is_local_text_candidate(input, modes) {
            continue;
        }
        groups
            .entry(PathBuf::from(input.source_path.as_str()))
            .or_default()
            .push((position, input.page_index));
    }

    for (source_path, page_requests) in groups {
        let page_indexes = page_requests
            .iter()
            .map(|(_, page_index)| *page_index)
            .collect::<Vec<_>>();
        let text_results = match extractor(source_path.as_path(), page_indexes.as_slice()) {
            Ok(texts) => texts,
            Err(error) => {
                fail_fast_local_backend_text_group(
                    &mut results,
                    inputs,
                    page_requests.as_slice(),
                    modes,
                    format!("local backend-text source extraction failed: {error}"),
                );
                continue;
            }
        };
        if text_results.len() != page_requests.len() {
            fail_fast_local_backend_text_group(
                &mut results,
                inputs,
                page_requests.as_slice(),
                modes,
                format!(
                    "local backend-text source extraction returned {} rows for {} requests",
                    text_results.len(),
                    page_requests.len()
                ),
            );
            continue;
        }
        for ((position, _), text_result) in page_requests.into_iter().zip(text_results) {
            let text = match text_result {
                Ok(text) => text,
                Err(error) => {
                    let reason = format!("local backend-text source extraction failed: {error}");
                    if let Some(result) = local_backend_text_fail_fast_result(
                        &inputs[position],
                        modes,
                        reason.as_str(),
                    ) {
                        results[position] = Some(result);
                    }
                    continue;
                }
            };
            if text.trim().is_empty() {
                if let Some(result) = local_backend_text_fail_fast_result(
                    &inputs[position],
                    modes,
                    "local backend-text returned empty text",
                ) {
                    results[position] = Some(result);
                }
                continue;
            }
            let mut result = PdfOcrShardResult::succeeded(&inputs[position], text, 1.0);
            result.text_mime_type = "text/markdown".to_string();
            results[position] = Some(result);
        }
    }

    results
}

fn fail_fast_local_backend_text_group(
    results: &mut [Option<PdfOcrShardResult>],
    inputs: &[PdfOcrShardInput],
    page_requests: &[(usize, u32)],
    modes: LocalTextModes,
    reason: String,
) {
    for (position, _) in page_requests {
        if let Some(result) =
            local_backend_text_fail_fast_result(&inputs[*position], modes, &reason)
        {
            results[*position] = Some(result);
        }
    }
}

fn local_backend_text_fail_fast_result(
    input: &PdfOcrShardInput,
    modes: LocalTextModes,
    reason: &str,
) -> Option<PdfOcrShardResult> {
    if modes.backend_text_empty != LocalBackendTextEmptyMode::FailFast {
        return None;
    }
    if input.ocr_profile != PDF_OCR_BACKEND_TEXT_PROFILE || !is_source_page_range_input(input) {
        return None;
    }
    Some(PdfOcrShardResult::failed(
        input,
        format!(
            "{reason} for source PDF page {}; source-page-range placeholder `{}` requires full-document fallback",
            input.page_index, input.image_path
        ),
    ))
}

fn is_source_page_range_input(input: &PdfOcrShardInput) -> bool {
    input.image_mime_type == SOURCE_PDF_PAGE_IMAGE_MIME_TYPE
        || input.image_path.ends_with(".source-page-range")
}

fn is_local_text_candidate(input: &PdfOcrShardInput, modes: LocalTextModes) -> bool {
    let profile_enabled = (modes.backend_text && input.ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE)
        || (modes.fast_text && input.ocr_profile == PDF_OCR_FAST_TEXT_PROFILE);
    profile_enabled
        && input.shard_type == "page"
        && !input.source_path.trim().is_empty()
        && Path::new(input.source_path.as_str()).is_file()
}

#[cfg(test)]
fn empty_text_extractor(
    _path: &Path,
    page_indexes: &[u32],
) -> Result<Vec<Result<String, String>>, String> {
    Ok(page_indexes.iter().map(|_| Ok(String::new())).collect())
}

#[cfg(test)]
fn error_text_extractor(
    _path: &Path,
    page_indexes: &[u32],
) -> Result<Vec<Result<String, String>>, String> {
    Ok(page_indexes
        .iter()
        .map(|_| Err("synthetic lopdf failure".to_string()))
        .collect())
}

#[cfg(test)]
fn partial_error_text_extractor(
    _path: &Path,
    page_indexes: &[u32],
) -> Result<Vec<Result<String, String>>, String> {
    Ok(page_indexes
        .iter()
        .map(|page_index| {
            if *page_index == 1 {
                Err("synthetic page failure".to_string())
            } else {
                Ok(format!("local page {page_index}"))
            }
        })
        .collect())
}
