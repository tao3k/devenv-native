use super::{DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS_ENV, pdf_ocr_endpoint_urls_with_lookup};

#[test]
fn pdf_ocr_endpoint_urls_default_to_document_extract_endpoint() {
    let endpoints = pdf_ocr_endpoint_urls_with_lookup("http://127.0.0.1:50051", &|_| None);

    assert_eq!(endpoints, vec!["http://127.0.0.1:50051"]);
}

#[test]
fn pdf_ocr_endpoint_urls_parse_pool_and_deduplicate() {
    let endpoints = pdf_ocr_endpoint_urls_with_lookup("http://default", &|key| {
        (key == DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS_ENV).then(|| {
            " http://127.0.0.1:50051/,http://127.0.0.1:50052; http://127.0.0.1:50051 ".to_string()
        })
    });

    assert_eq!(
        endpoints,
        vec!["http://127.0.0.1:50051", "http://127.0.0.1:50052"]
    );
}

#[test]
fn pdf_ocr_endpoint_urls_fall_back_when_config_is_empty() {
    let endpoints = pdf_ocr_endpoint_urls_with_lookup("http://fallback/", &|key| {
        (key == DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS_ENV).then(|| " , ; ".to_string())
    });

    assert_eq!(endpoints, vec!["http://fallback"]);
}
