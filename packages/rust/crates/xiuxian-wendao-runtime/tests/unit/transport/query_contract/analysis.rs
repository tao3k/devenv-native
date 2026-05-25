use super::{
    ANALYSIS_AUDIO_SHARDS_ROUTE, ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE, ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    WENDAO_AUDIO_HOSTED_BASE_URL_HEADER, WENDAO_AUDIO_HOSTED_MODEL_HEADER,
    WENDAO_AUDIO_HOSTED_PROVIDER_HEADER, WENDAO_AUDIO_WORKER_HEADER, WENDAO_AUDIO_WORKERS_HEADER,
    WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER, WENDAO_DOCUMENT_EXTRACT_MODE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER, WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER,
    WENDAO_PDF_OCR_WORKERS_HEADER, decode_document_extract_source_path_utf8_hex,
    encode_document_extract_source_path_utf8_hex, validate_code_ast_analysis_request,
    validate_document_extract_request, validate_markdown_analysis_request,
};

#[test]
fn markdown_analysis_request_validation_accepts_stable_request() {
    assert!(validate_markdown_analysis_request("docs/analysis.md").is_ok());
}

#[test]
fn markdown_analysis_request_validation_rejects_blank_path() {
    assert_eq!(
        validate_markdown_analysis_request("   "),
        Err("markdown analysis path must not be blank".to_string())
    );
}

#[test]
fn code_ast_analysis_request_validation_accepts_stable_request() {
    assert!(validate_code_ast_analysis_request("src/lib.jl", "demo", Some(7)).is_ok());
}

#[test]
fn code_ast_analysis_request_validation_rejects_blank_repo() {
    assert_eq!(
        validate_code_ast_analysis_request("src/lib.jl", "   ", Some(7)),
        Err("code AST analysis repo must not be blank".to_string())
    );
}

#[test]
fn code_ast_analysis_request_validation_rejects_zero_line_hint() {
    assert_eq!(
        validate_code_ast_analysis_request("src/lib.jl", "demo", Some(0)),
        Err("code AST analysis line hint must be greater than zero".to_string())
    );
}

#[test]
fn document_extract_contract_uses_document_route_and_headers() {
    assert_eq!(
        ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
        "/analysis/document-extract"
    );
    assert_eq!(
        WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
        "x-wendao-document-extract-source-path"
    );
    assert_eq!(
        WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER,
        "x-wendao-document-extract-source-path-utf8-hex"
    );
    assert_eq!(
        WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
        "x-wendao-document-extract-output-dir"
    );
    assert_eq!(
        WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
        "x-wendao-document-extract-profile"
    );
    assert_eq!(
        ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE,
        "/analysis/document-extract-status"
    );
    assert_eq!(ANALYSIS_PDF_OCR_SHARDS_ROUTE, "/analysis/pdf-ocr-shards");
    assert_eq!(ANALYSIS_AUDIO_SHARDS_ROUTE, "/analysis/audio-shards");
    assert_eq!(
        WENDAO_DOCUMENT_EXTRACT_MODE_HEADER,
        "x-wendao-document-extract-mode"
    );
    assert_eq!(
        WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER,
        "x-wendao-document-extract-wait-ms"
    );
    assert_eq!(
        WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER,
        "x-wendao-document-extract-job-id"
    );
    assert_eq!(WENDAO_PDF_OCR_WORKERS_HEADER, "x-wendao-pdf-ocr-workers");
    assert_eq!(WENDAO_AUDIO_WORKERS_HEADER, "x-wendao-audio-workers");
    assert_eq!(WENDAO_AUDIO_WORKER_HEADER, "x-wendao-audio-worker");
    assert_eq!(
        WENDAO_AUDIO_HOSTED_PROVIDER_HEADER,
        "x-wendao-audio-hosted-provider"
    );
    assert_eq!(
        WENDAO_AUDIO_HOSTED_BASE_URL_HEADER,
        "x-wendao-audio-hosted-base-url"
    );
    assert_eq!(
        WENDAO_AUDIO_HOSTED_MODEL_HEADER,
        "x-wendao-audio-hosted-model"
    );
}

#[test]
fn document_extract_request_validation_accepts_stable_request() {
    assert!(validate_document_extract_request("docs/manual.pdf").is_ok());
}

#[test]
fn document_extract_source_path_utf8_hex_roundtrips_non_ascii_paths() {
    let source_path = "private-fixtures/audio-\u{97f3}\u{9891}.mp3";

    let encoded = encode_document_extract_source_path_utf8_hex(source_path);

    assert!(encoded.is_ascii());
    assert_eq!(
        decode_document_extract_source_path_utf8_hex(encoded.as_str()),
        Ok(source_path.to_string())
    );
}

#[test]
fn document_extract_request_validation_rejects_blank_path() {
    assert_eq!(
        validate_document_extract_request("   "),
        Err("document extract source path must not be blank".to_string())
    );
}
