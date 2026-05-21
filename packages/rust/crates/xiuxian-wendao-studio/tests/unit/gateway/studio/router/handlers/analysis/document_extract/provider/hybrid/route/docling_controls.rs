#[test]
fn docling_page_range_chunk_size_accepts_only_positive_integers() {
    assert_eq!(
        docling_page_range_chunk_size_with_lookup(&|_key| None),
        None
    );
    assert_eq!(
        docling_page_range_chunk_size_with_lookup(&|_key| Some("3".to_string())),
        Some(3)
    );
    assert_eq!(
        docling_page_range_chunk_size_with_lookup(&|_key| Some("0".to_string())),
        None
    );
    assert_eq!(
        docling_page_range_chunk_size_with_lookup(&|_key| Some("invalid".to_string())),
        None
    );
}

#[test]
fn docling_page_range_profile_accepts_only_structure_text_override() {
    assert_eq!(
        docling_page_range_fallback_profile_with_lookup(&|_key| None),
        "full"
    );
    assert_eq!(
        docling_page_range_fallback_profile_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE_ENV)
                .then(|| "docling-structure-text".to_string())
        }),
        "structure-text"
    );
    assert_eq!(
        docling_page_range_fallback_profile_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE_ENV)
                .then(|| "fast-text".to_string())
        }),
        "full"
    );
}

#[test]
fn docling_page_range_chunk_count_uses_document_extract_endpoint_pool() {
    let count = docling_page_range_document_extract_endpoint_count_with_lookup(
        Some("http://default:50051"),
        &|key| {
            (key == "WENDAO_DOCUMENT_EXTRACT_ENDPOINTS").then(|| {
                " http://one:50051/,http://two:50051;http://one:50051 ".to_string()
            })
        },
    );

    assert_eq!(count, 2);
}

#[test]
fn docling_page_range_chunk_concurrency_accepts_only_positive_integers() {
    assert_eq!(
        docling_page_range_chunk_concurrency_with_lookup(&|_key| None),
        None
    );
    assert_eq!(
        docling_page_range_chunk_concurrency_with_lookup(&|_key| Some("2".to_string())),
        Some(2)
    );
    assert_eq!(
        docling_page_range_chunk_concurrency_with_lookup(&|_key| Some("0".to_string())),
        None
    );
    assert_eq!(
        docling_page_range_chunk_concurrency_with_lookup(&|_key| Some("invalid".to_string())),
        None
    );
}

#[test]
fn docling_page_range_chunk_concurrency_default_follows_endpoint_pool() {
    assert_eq!(
        docling_page_range_chunk_concurrency_limit_with_lookup(6, 4, &|_key| None),
        4
    );
    assert_eq!(
        docling_page_range_chunk_concurrency_limit_with_lookup(2, 4, &|_key| None),
        2
    );
    assert_eq!(
        docling_page_range_chunk_concurrency_limit_with_lookup(6, 4, &|_key| {
            Some("5".to_string())
        }),
        5
    );
}

#[test]
fn docling_page_range_hedge_delay_accepts_only_positive_milliseconds() {
    assert_eq!(
        docling_page_range_hedge_delay_ms_with_lookup(&|_key| None),
        None
    );
    assert_eq!(
        docling_page_range_hedge_delay_ms_with_lookup(&|_key| Some("7000".to_string())),
        Some(7000)
    );
    assert_eq!(
        docling_page_range_hedge_delay_ms_with_lookup(&|_key| Some("0".to_string())),
        None
    );
    assert_eq!(
        docling_page_range_hedge_delay_ms_with_lookup(&|_key| Some("invalid".to_string())),
        None
    );
}
