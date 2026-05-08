use super::{
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE, endpoint_index_for_request, rendered_region_shard_chunks,
    rendered_region_shard_chunks_with_composite_size, sample_ocr_input,
    source_pdf_page_range_chunk_endpoint_index_with_lookup,
    source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup, source_pdf_page_range_chunks,
    source_pdf_page_range_chunks_with_fast_text_split, source_pdf_page_range_chunks_with_weights,
    source_pdf_page_range_dispatch_budget,
    source_pdf_page_range_dispatch_budget_with_region_pipeline,
    source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split,
    source_pdf_page_range_dispatch_chunks,
};

#[test]
fn source_pdf_page_range_chunks_split_balanced_contiguous_ranges() {
    let inputs = (0..21)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 4);

    assert_eq!(chunks.len(), 4);
    assert_eq!(chunks[0].len(), 6);
    assert_eq!(chunks[1].len(), 5);
    assert_eq!(chunks[2].len(), 5);
    assert_eq!(chunks[3].len(), 5);
    assert_eq!(chunks[0][0].page_index, 0);
    assert_eq!(chunks[0][5].page_index, 5);
    assert_eq!(chunks[1][0].page_index, 6);
    assert_eq!(chunks[3][4].page_index, 20);
}

#[test]
fn source_pdf_page_range_chunks_keep_single_range_for_one_permit() {
    let inputs = (0..3)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 1);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].len(), 3);
    assert_eq!(chunks[0][0].page_index, 0);
    assert_eq!(chunks[0][2].page_index, 2);
}

#[test]
fn source_pdf_page_range_chunks_do_not_cross_cache_miss_gaps() {
    let inputs = [0, 1, 4, 5, 8]
        .into_iter()
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 2);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs, vec![vec![0, 1], vec![4, 5], vec![8]]);
}

#[test]
fn source_pdf_page_range_chunks_split_long_runs_without_crossing_gaps() {
    let inputs = (0..9)
        .chain(20..29)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 4);

    assert_eq!(chunks.len(), 4);
    for chunk in chunks {
        for window in chunk.windows(2) {
            assert_eq!(window[1].page_index, window[0].page_index + 1);
        }
    }
}

#[test]
fn source_pdf_page_range_chunks_with_weights_preserve_order_and_isolate_heavy_pages() {
    let inputs = (0..9)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    let weights = [1, 1, 1, 1, 20, 1, 1, 1, 1];

    let chunks = source_pdf_page_range_chunks_with_weights(inputs.as_slice(), 3, &weights);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs, vec![vec![0, 1, 2, 3], vec![4], vec![5, 6, 7, 8]]);
}

#[test]
fn source_pdf_page_range_chunks_with_weights_do_not_cross_cache_miss_gaps() {
    let inputs = [0, 1, 4, 5, 8]
        .into_iter()
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    let weights = [1, 30, 1, 1, 1];

    let chunks = source_pdf_page_range_chunks_with_weights(inputs.as_slice(), 2, &weights);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs, vec![vec![0, 1], vec![4, 5], vec![8]]);
}

#[test]
fn source_pdf_page_range_chunks_do_not_cross_ocr_profile_boundaries() {
    let mut inputs = (0..6)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    for input in &mut inputs[2..4] {
        input.ocr_profile = "docling-fast-text-ocr".to_string();
    }

    let chunks = source_pdf_page_range_chunks(inputs.as_slice(), 2);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs, vec![vec![0, 1], vec![2, 3], vec![4, 5]]);
}

#[test]
fn source_pdf_page_range_dispatch_chunks_prioritize_topup_profiles_for_backend_text_mode() {
    let mut inputs = (0..8)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    for input in &mut inputs {
        input.ocr_profile = "docling-backend-text-ocr-v1".to_string();
        input.raster_width_px = 10;
        input.raster_height_px = 10;
    }
    inputs[2].ocr_profile = "docling-fast-text-ocr".to_string();
    inputs[2].raster_width_px = 100;
    inputs[2].raster_height_px = 100;
    inputs[5].ocr_profile = "docling-fast-text-ocr".to_string();
    inputs[5].raster_width_px = 80;
    inputs[5].raster_height_px = 80;

    let chunks = source_pdf_page_range_dispatch_chunks(inputs.as_slice(), 3);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs[0], vec![2]);
    assert_eq!(page_runs[1], vec![5]);
}

#[test]
fn source_pdf_page_range_dispatch_budget_expands_to_backend_text_profile_runs() {
    let mut inputs = (0..8)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    for input in &mut inputs {
        input.ocr_profile = "docling-backend-text-ocr-v1".to_string();
    }
    inputs[2].ocr_profile = "docling-fast-text-ocr".to_string();
    inputs[5].ocr_profile = "docling-fast-text-ocr".to_string();

    assert_eq!(
        source_pdf_page_range_dispatch_budget(inputs.as_slice(), 3),
        5
    );
    assert_eq!(
        source_pdf_page_range_dispatch_budget(inputs.as_slice(), 9),
        8
    );

    for input in &mut inputs {
        input.ocr_profile = "docling-fast-text-ocr".to_string();
    }

    assert_eq!(
        source_pdf_page_range_dispatch_budget(inputs.as_slice(), 3),
        3
    );
}

#[test]
fn source_pdf_page_range_dispatch_budget_expands_to_runs_during_region_pipeline() {
    let mut inputs = (0..8)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    for input in &mut inputs {
        input.ocr_profile = "docling-backend-text-ocr-v1".to_string();
    }
    inputs[2].ocr_profile = "docling-fast-text-ocr".to_string();
    inputs[5].ocr_profile = "docling-fast-text-ocr".to_string();

    assert_eq!(
        source_pdf_page_range_dispatch_budget_with_region_pipeline(inputs.as_slice(), 3, true),
        5
    );
    assert_eq!(
        source_pdf_page_range_dispatch_budget_with_region_pipeline(inputs.as_slice(), 9, true),
        8
    );

    for input in &mut inputs {
        input.ocr_profile = "docling-fast-text-ocr".to_string();
    }
    inputs[0].page_index = 5;
    inputs[1].page_index = 11;
    inputs[2].page_index = 12;
    inputs[3].page_index = 13;
    inputs.truncate(4);

    assert_eq!(
        source_pdf_page_range_dispatch_budget_with_region_pipeline(inputs.as_slice(), 1, true),
        2
    );
    assert_eq!(
        source_pdf_page_range_dispatch_budget_with_region_pipeline(inputs.as_slice(), 1, false),
        1
    );
}

#[test]
fn source_pdf_page_range_fast_text_split_uses_single_page_chunks_when_enabled() {
    let mut inputs = (0..4)
        .map(|page_index| sample_ocr_input("/tmp/source.pdf", page_index, "page"))
        .collect::<Vec<_>>();
    for input in &mut inputs {
        input.ocr_profile = "docling-fast-text-ocr".to_string();
    }
    inputs[0].page_index = 5;
    inputs[1].page_index = 11;
    inputs[2].page_index = 12;
    inputs[3].page_index = 13;

    assert_eq!(
        source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split(
            inputs.as_slice(),
            1,
            true,
            false,
        ),
        2
    );
    assert_eq!(
        source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split(
            inputs.as_slice(),
            1,
            true,
            true,
        ),
        2
    );
    assert_eq!(
        source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split(
            inputs.as_slice(),
            1,
            false,
            true,
        ),
        1
    );

    let chunks = source_pdf_page_range_chunks_with_fast_text_split(inputs.as_slice(), 1, true);
    let page_runs = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| input.page_index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(page_runs, vec![vec![5], vec![11], vec![12], vec![13]]);
}

#[test]
fn rendered_region_shard_chunks_start_dense_risk_pages_before_largest_single_region() {
    let mut inputs = (0..6)
        .map(|region_index| {
            let mut input = sample_ocr_input("/tmp/source.pdf", 12, "region");
            input.region_index = region_index;
            input.shard_element_id = format!("region-{region_index}");
            input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
            input.ocr_engine = "hosted-vlm-direct-ocr".to_string();
            input
        })
        .collect::<Vec<_>>();
    inputs[4].page_index = 13;
    inputs[5].page_index = 13;
    let widths = [100, 500, 200, 400, 900, 300];
    for (input, width) in inputs.iter_mut().zip(widths) {
        input.source_page_pixel_left = 0;
        input.source_page_pixel_right = width;
        input.source_page_pixel_top = 0;
        input.source_page_pixel_bottom = 100;
    }

    let chunks = rendered_region_shard_chunks(inputs.as_slice());
    let chunk_regions = chunks
        .iter()
        .map(|chunk| {
            assert_eq!(chunk.len(), 1);
            (chunk[0].page_index, chunk[0].region_index)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        chunk_regions,
        vec![(12, 1), (12, 3), (12, 2), (12, 0), (13, 4), (13, 5)]
    );
}

#[test]
fn rendered_region_shard_chunks_group_same_page_regions_for_composite_canary() {
    let mut inputs = (0..6)
        .map(|region_index| {
            let mut input = sample_ocr_input("/tmp/source.pdf", 12, "region");
            input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
            input.shard_type = "region".to_string();
            input.region_index = region_index;
            input.reading_order_key = format!("000012.{region_index:06}");
            input.parent_shard_element_id = "parent-page-12".to_string();
            input
        })
        .collect::<Vec<_>>();
    inputs[3].page_index = 13;
    inputs[3].reading_order_key = "000013.000003".to_string();
    inputs[3].parent_shard_element_id = "parent-page-13".to_string();
    inputs[4].page_index = 13;
    inputs[4].reading_order_key = "000013.000004".to_string();
    inputs[4].parent_shard_element_id = "parent-page-13".to_string();
    inputs[5].page_index = 13;
    inputs[5].reading_order_key = "000013.000005".to_string();
    inputs[5].parent_shard_element_id = "parent-page-13".to_string();

    let chunks = rendered_region_shard_chunks_with_composite_size(inputs.as_slice(), 2);
    let chunk_regions = chunks
        .iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|input| (input.page_index, input.region_index))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        chunk_regions,
        vec![
            vec![(12, 0), (12, 1)],
            vec![(13, 3), (13, 4)],
            vec![(12, 2)],
            vec![(13, 5)]
        ]
    );
}

#[test]
fn endpoint_index_for_request_round_robins_endpoint_pool() -> Result<(), String> {
    assert_eq!(endpoint_index_for_request(0, 3)?, 0);
    assert_eq!(endpoint_index_for_request(1, 3)?, 1);
    assert_eq!(endpoint_index_for_request(2, 3)?, 2);
    assert_eq!(endpoint_index_for_request(3, 3)?, 0);
    assert!(endpoint_index_for_request(0, 0).is_err());
    Ok(())
}

#[test]
fn source_pdf_page_range_endpoint_affinity_targets_single_fast_text_pdf_page() {
    let mut input = sample_ocr_input("/tmp/source.pdf", 5, "page");
    input.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let enabled = |key: &str| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY")
            .then(|| "single-page-first".to_string())
    };

    assert!(
        source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(
            std::slice::from_ref(&input),
            &enabled,
        )
    );

    let disabled = |_: &str| None;
    assert!(
        !source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(
            std::slice::from_ref(&input),
            &disabled,
        )
    );
}

#[test]
fn source_pdf_page_range_endpoint_affinity_routes_single_fast_text_chunk_to_first_endpoint() {
    let mut input = sample_ocr_input("/tmp/source.pdf", 5, "page");
    input.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let enabled = |key: &str| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY")
            .then(|| "single-page-first".to_string())
    };

    let endpoint_index = source_pdf_page_range_chunk_endpoint_index_with_lookup(
        4,
        std::slice::from_ref(&input),
        &enabled,
        || Err("affinity should not advance the round-robin cursor".to_string()),
    )
    .expect("single fast-text source chunk should resolve");

    assert_eq!(endpoint_index, 0);
    assert!(
        source_pdf_page_range_chunk_endpoint_index_with_lookup(0, &[input], &enabled, || Err(
            "affinity should not advance the round-robin cursor".to_string()
        ),)
        .is_err()
    );
}

#[test]
fn source_pdf_page_range_endpoint_affinity_uses_round_robin_for_other_chunks() {
    let mut first = sample_ocr_input("/tmp/source.pdf", 11, "page");
    let mut second = sample_ocr_input("/tmp/source.pdf", 12, "page");
    first.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    second.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let enabled = |key: &str| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY")
            .then(|| "single-page-first".to_string())
    };

    let endpoint_index = source_pdf_page_range_chunk_endpoint_index_with_lookup(
        4,
        &[first, second],
        &enabled,
        || Ok(2),
    )
    .expect("multi-page fast-text source chunk should use round-robin");

    assert_eq!(endpoint_index, 2);
}

#[test]
fn source_pdf_page_range_endpoint_affinity_rejects_non_single_fast_text_pdf_chunks() {
    let mut fast_page = sample_ocr_input("/tmp/source.pdf", 5, "page");
    fast_page.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let mut second_fast_page = sample_ocr_input("/tmp/source.pdf", 6, "page");
    second_fast_page.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let mut region = sample_ocr_input("/tmp/source.pdf", 5, "region");
    region.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let mut png_source = sample_ocr_input("/tmp/source.png", 5, "page");
    png_source.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let enabled = |key: &str| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY")
            .then(|| "single-page-first".to_string())
    };

    assert!(
        !source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(
            &[fast_page, second_fast_page],
            &enabled,
        )
    );
    assert!(!source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(&[region], &enabled,));
    assert!(
        !source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(&[png_source], &enabled,)
    );
}
