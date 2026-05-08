use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PdfOcrShardInput, PdfOcrShardResult,
};
use xiuxian_wendao_attachments::pdf::text::source_pdf_page_texts;

pub(crate) const DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT";
pub(crate) const DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT";
const LOCAL_TEXT_RUST_LOPDF_MODE: &str = "rust-lopdf";

#[derive(Debug, Clone, Copy)]
struct LocalTextModes {
    backend_text: bool,
    fast_text: bool,
}

impl LocalTextModes {
    fn from_env() -> Self {
        Self {
            backend_text: local_text_env_enabled(DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_ENV),
            fast_text: local_text_env_enabled(DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT_ENV),
        }
    }

    #[cfg(test)]
    fn backend_text_only() -> Self {
        Self {
            backend_text: true,
            fast_text: false,
        }
    }

    #[cfg(test)]
    fn backend_and_fast_text() -> Self {
        Self {
            backend_text: true,
            fast_text: true,
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

fn local_text_env_enabled(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| {
            value.trim().replace('_', "-").to_ascii_lowercase() == LOCAL_TEXT_RUST_LOPDF_MODE
        })
        .unwrap_or(false)
}

fn local_text_results_enabled(
    inputs: &[PdfOcrShardInput],
    modes: LocalTextModes,
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
        let Ok(texts) = source_pdf_page_texts(source_path.as_path(), page_indexes.as_slice())
        else {
            continue;
        };
        if texts.len() != page_requests.len() {
            continue;
        }
        for ((position, _), text) in page_requests.into_iter().zip(texts) {
            if text.trim().is_empty() {
                continue;
            }
            let mut result = PdfOcrShardResult::succeeded(&inputs[position], text, 1.0);
            result.text_mime_type = "text/markdown".to_string();
            results[position] = Some(result);
        }
    }

    results
}

fn is_local_text_candidate(input: &PdfOcrShardInput, modes: LocalTextModes) -> bool {
    let profile_enabled = (modes.backend_text && input.ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE)
        || (modes.fast_text && input.ocr_profile == PDF_OCR_FAST_TEXT_PROFILE);
    profile_enabled
        && input.shard_type == "page"
        && !input.source_path.trim().is_empty()
        && Path::new(input.source_path.as_str()).is_file()
}
