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
        4
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
