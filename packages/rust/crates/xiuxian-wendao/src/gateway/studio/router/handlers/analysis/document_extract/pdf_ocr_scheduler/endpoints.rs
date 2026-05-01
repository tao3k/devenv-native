use std::collections::HashSet;

pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS";

pub(crate) fn pdf_ocr_endpoint_urls(default_endpoint: &str) -> Vec<String> {
    pdf_ocr_endpoint_urls_with_lookup(default_endpoint, &|key| std::env::var(key).ok())
}

pub(crate) fn pdf_ocr_endpoint_urls_with_lookup(
    default_endpoint: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<String> {
    let configured = lookup(DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS_ENV)
        .unwrap_or_else(|| default_endpoint.to_string());
    let mut seen = HashSet::new();
    let endpoints = configured
        .split(|character: char| character == ',' || character == ';' || character.is_whitespace())
        .filter_map(normalize_endpoint)
        .filter(|endpoint| seen.insert(endpoint.clone()))
        .collect::<Vec<_>>();
    if endpoints.is_empty() {
        normalize_endpoint(default_endpoint)
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        endpoints
    }
}

fn normalize_endpoint(endpoint: &str) -> Option<String> {
    let endpoint = endpoint.trim().trim_end_matches('/');
    (!endpoint.is_empty()).then(|| endpoint.to_string())
}

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/endpoints.rs"]
mod tests;
