#[test]
fn docling_page_range_wrapper_normalization_merges_chunk_document_rows() -> Result<(), String> {
    let batch = sample_document_resource_batch(&[
        ("document", 0, "page 1 markdown", "pages-00001-00003_main"),
        (
            "docling_json",
            0,
            "{\"chunk\":1}",
            "pages-00001-00003_docling_json",
        ),
        ("table", 0, "table one", "pages-00001-00003_table_000001"),
        ("document", 3, "page 4 markdown", "pages-00004-00006_main"),
        (
            "docling_json",
            3,
            "{\"chunk\":2}",
            "pages-00004-00006_docling_json",
        ),
        ("image", 3, "image one", "pages-00004-00006_image_000001"),
    ])?;

    let normalized = normalize_docling_page_range_wrapper_rows(batch)?;

    assert_eq!(normalized.num_rows(), 4);
    assert_eq!(
        test_string_value(&normalized, "resourceType", 0)?,
        "document"
    );
    assert_eq!(
        test_string_value(&normalized, "content", 0)?,
        "page 1 markdown\n\npage 4 markdown"
    );
    assert_eq!(
        test_string_value(&normalized, "elementId", 0)?,
        "pages-00001-00003_main"
    );
    assert_eq!(
        test_string_value(&normalized, "resourceType", 1)?,
        "docling_json"
    );
    assert_eq!(
        test_string_value(&normalized, "content", 1)?,
        "{\"chunk\":1}"
    );
    assert_eq!(test_string_value(&normalized, "resourceType", 2)?, "table");
    assert_eq!(test_string_value(&normalized, "resourceType", 3)?, "image");
    Ok(())
}

#[test]
fn docling_page_range_fallback_phase_is_not_reported_as_scheduler() -> Result<(), String> {
    let batch = HybridDocumentResourceBatch::native(sample_document_resource_batch(&[(
        "document", 0, "markdown", "main",
    )])?)
    .with_page_range_docling_fallback_pages(vec![0]);
    let mut phases = std::collections::BTreeMap::new();

    record_ocr_scheduler_or_docling_fallback_phase(&mut phases, &batch, Instant::now());

    assert_eq!(phases["ocrScheduler"], 0.0);
    assert!(phases.contains_key("doclingPageRangeFallback"));
    Ok(())
}

#[test]
fn docling_page_range_fallback_with_kept_ocr_reports_both_phases() -> Result<(), String> {
    let input = sample_page_input(1, PDF_OCR_BACKEND_TEXT_PROFILE, "docling-backend-text-ocr");
    let result = PdfOcrShardResult::succeeded(&input, "local backend text", 1.0);
    let metric = xiuxian_wendao_attachments::pdf::metrics::PdfOcrShardMetric::from_ocr_result(
        &input,
        &result,
        2,
        Some(42.0),
    );
    let batch = HybridDocumentResourceBatch::new(
        sample_document_resource_batch(&[("document", 0, "markdown", "main")])?,
        vec![input],
        vec![result],
        vec![metric],
        2,
        vec![1],
    )
    .with_page_range_docling_fallback_pages(vec![0]);
    let mut phases = std::collections::BTreeMap::new();

    record_ocr_scheduler_or_docling_fallback_phase(&mut phases, &batch, Instant::now());

    assert_eq!(phases["ocrScheduler"], 42.0);
    assert!(phases.contains_key("doclingPageRangeFallback"));
    Ok(())
}

#[test]
fn docling_page_range_fallback_chunk_summary_tracks_longest_chunk() {
    let chunks = vec![
        PageRangeDoclingFallbackChunkTiming {
            page_start: 0,
            page_end: 2,
            one_based_start: 1,
            one_based_end: 3,
            elapsed_ms: 1200.0,
            resource_rows: 9,
            document_extract_profile: "full".to_string(),
            hedged: false,
            attempt_count: 1,
            hedge_delay_ms: None,
            document_timing_total_elapsed_ms: Some(1100.0),
            document_timing_phase_elapsed_ms: std::collections::BTreeMap::from([
                ("doclingConvert".to_string(), 900.0),
                ("total".to_string(), 1100.0),
            ]),
            source_profile: Some(PageRangeDoclingFallbackSourceProfileSummary {
                page_count: 3,
                estimated_weight_total: 30,
                estimated_weight_max: 12,
                content_bytes_total: 300,
                operation_count_total: 120,
                text_show_ops_total: 90,
                path_ops_total: 12,
                rectangle_ops_total: 2,
                draw_object_ops_total: 1,
                structure_authority_required_count: 1,
                fast_profile_risk_count: 0,
                backend_text_topup_count: 0,
            }),
        },
        PageRangeDoclingFallbackChunkTiming {
            page_start: 3,
            page_end: 5,
            one_based_start: 4,
            one_based_end: 6,
            elapsed_ms: 3400.0,
            resource_rows: 12,
            document_extract_profile: "structure-text".to_string(),
            hedged: true,
            attempt_count: 2,
            hedge_delay_ms: Some(7000),
            document_timing_total_elapsed_ms: Some(3200.0),
            document_timing_phase_elapsed_ms: std::collections::BTreeMap::from([
                ("doclingConvert".to_string(), 3000.0),
                ("total".to_string(), 3200.0),
            ]),
            source_profile: Some(PageRangeDoclingFallbackSourceProfileSummary {
                page_count: 3,
                estimated_weight_total: 60,
                estimated_weight_max: 30,
                content_bytes_total: 600,
                operation_count_total: 240,
                text_show_ops_total: 150,
                path_ops_total: 80,
                rectangle_ops_total: 5,
                draw_object_ops_total: 2,
                structure_authority_required_count: 2,
                fast_profile_risk_count: 1,
                backend_text_topup_count: 1,
            }),
        },
    ];

    let summary = page_range_docling_fallback_chunk_summary(chunks.as_slice());

    assert_eq!(summary["chunkCount"], 2);
    assert_eq!(summary["resourceRows"], 21);
    assert_eq!(summary["longestPageStart"], 3);
    assert_eq!(summary["longestPageEnd"], 5);
    assert_eq!(summary["longestResourceRows"], 12);
    assert_eq!(summary["hedgedChunkCount"], 1);
    assert_eq!(summary["attemptCountTotal"], 3);
    assert_eq!(summary["elapsedMsMax"], 3400.0);
    assert_eq!(summary["elapsedMsMin"], 1200.0);
    assert_eq!(summary["elapsedMsMean"], 2300.0);
    assert_eq!(summary["elapsedMsSpread"], 2200.0);
    assert_eq!(summary["elapsedMsMaxToMeanRatio"], 3400.0 / 2300.0);
    assert_eq!(summary["elapsedMsTotal"], 4600.0);
    assert_eq!(summary["documentTimingTotalElapsedMs"], 4300.0);
    assert_eq!(
        summary["documentTimingPhaseElapsedMs"]["doclingConvert"],
        3900.0
    );
    assert_eq!(summary["documentExtractProfileCounts"]["full"], 1);
    assert_eq!(
        summary["documentExtractProfileCounts"]["structure-text"],
        1
    );
    assert_eq!(summary["longestDocumentTimingTotalElapsedMs"], 3200.0);
    assert_eq!(
        summary["longestDocumentTimingPhaseElapsedMs"]["doclingConvert"],
        3000.0
    );
    assert_eq!(summary["sourceProfilePageCount"], 6);
    assert_eq!(summary["sourceProfileEstimatedWeightTotal"], 90);
    assert_eq!(summary["sourceProfileStructureAuthorityRequiredCount"], 3);
    assert_eq!(summary["sourceProfileFastProfileRiskCount"], 1);
    assert_eq!(summary["sourceProfileBackendTextTopupCount"], 1);
    assert_eq!(summary["longestSourceProfile"]["estimatedWeightTotal"], 60);
    assert_eq!(summary["longestSourceProfile"]["fastProfileRiskCount"], 1);
}

#[test]
fn docling_centered_structure_count_includes_page_range_source_profiles() -> Result<(), String> {
    let default_page = sample_page_input(0, PDF_OCR_DEFAULT_PROFILE, "docling-compatible-ocr");
    let batch = HybridDocumentResourceBatch::with_ocr(
        sample_document_resource_batch(&[("document", 0, "markdown", "main")])?,
        vec![default_page],
        Vec::new(),
    )
    .with_page_range_docling_fallback_chunks(vec![PageRangeDoclingFallbackChunkTiming {
        page_start: 1,
        page_end: 3,
        one_based_start: 2,
        one_based_end: 4,
        elapsed_ms: 1000.0,
        resource_rows: 4,
        document_extract_profile: "full".to_string(),
        hedged: false,
        attempt_count: 1,
        hedge_delay_ms: None,
        document_timing_total_elapsed_ms: None,
        document_timing_phase_elapsed_ms: std::collections::BTreeMap::new(),
        source_profile: Some(PageRangeDoclingFallbackSourceProfileSummary {
            page_count: 3,
            estimated_weight_total: 42,
            estimated_weight_max: 20,
            content_bytes_total: 4096,
            operation_count_total: 200,
            text_show_ops_total: 100,
            path_ops_total: 80,
            rectangle_ops_total: 0,
            draw_object_ops_total: 1,
            structure_authority_required_count: 2,
            fast_profile_risk_count: 1,
            backend_text_topup_count: 0,
        }),
    }]);

    assert_eq!(docling_centered_structure_authority_page_count(&batch), 3);

    Ok(())
}
