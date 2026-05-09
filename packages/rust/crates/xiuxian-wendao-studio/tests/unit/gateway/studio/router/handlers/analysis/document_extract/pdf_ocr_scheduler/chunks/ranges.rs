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
