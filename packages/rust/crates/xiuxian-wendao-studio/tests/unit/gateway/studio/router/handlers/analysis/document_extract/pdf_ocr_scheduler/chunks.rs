use super::{
    PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE, endpoint_index_for_request,
    rendered_region_shard_chunks, rendered_region_shard_chunks_with_composite_size,
    sample_ocr_input, source_pdf_page_range_chunks, source_pdf_page_range_chunks_with_weights,
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
fn rendered_region_shard_chunks_start_largest_regions_first_for_tail_control() {
    let mut inputs = (0..6)
        .map(|region_index| {
            let mut input = sample_ocr_input("/tmp/source.pdf", 12, "region");
            input.region_index = region_index;
            input.shard_element_id = format!("region-{region_index}");
            input.ocr_profile = PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE.to_string();
            input.ocr_engine = "deepseek-ocr2-direct-vlm".to_string();
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
        vec![(13, 4), (12, 1), (12, 3), (13, 5), (12, 2), (12, 0)]
    );
}

#[test]
fn rendered_region_shard_chunks_group_same_page_regions_for_composite_canary() {
    let mut inputs = (0..6)
        .map(|region_index| {
            let mut input = sample_ocr_input("/tmp/source.pdf", 12, "region");
            input.ocr_profile = PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE.to_string();
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
