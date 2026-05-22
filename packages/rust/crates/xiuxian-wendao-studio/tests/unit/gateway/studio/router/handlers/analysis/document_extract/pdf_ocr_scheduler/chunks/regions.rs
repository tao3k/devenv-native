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
fn rendered_region_dispatch_chunk_size_uses_rust_dispatch_env_only() {
    assert_eq!(
        rendered_region_dispatch_chunk_size_with_lookup(&|key| {
            (key == "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE").then(|| "3".to_string())
        }),
        1
    );
    assert_eq!(
        rendered_region_dispatch_chunk_size_with_lookup(&|key| {
            (key == HOSTED_VLM_REGION_DISPATCH_CHUNK_SIZE_ENV).then(|| "3".to_string())
        }),
        3
    );
}
