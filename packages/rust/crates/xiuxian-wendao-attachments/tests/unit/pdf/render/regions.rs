use super::{PdfPageRegionRenderRequest, page_region_render_request_chunks_all};
use crate::pdf::render::{PdfPageBox, page_region_render_request_chunks_by_region_seed_page};

#[test]
fn page_region_render_request_chunks_all_preserve_reading_order_in_one_chunk() {
    let chunks = page_region_render_request_chunks_all(&[
        PdfPageRegionRenderRequest::new(
            2,
            2,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000002.000002".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            1,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000001.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            2,
            1,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000002.000001".to_string()),
        ),
    ]);

    assert_eq!(chunks.len(), 1);
    assert_eq!(
        chunks[0]
            .iter()
            .map(|region| (region.page_index, region.region_index))
            .collect::<Vec<_>>(),
        vec![(1, 1), (2, 1), (2, 2)]
    );
}

#[test]
fn page_region_render_request_chunks_by_region_seed_page_starts_with_smallest_region() {
    let chunks = page_region_render_request_chunks_by_region_seed_page(&[
        PdfPageRegionRenderRequest::new(
            2,
            1,
            PdfPageBox::new(0.0, 0.0, 30.0, 30.0),
            Some("000002.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            2,
            PdfPageBox::new(0.0, 0.0, 10.0, 10.0),
            Some("000001.000002".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            1,
            PdfPageBox::new(0.0, 0.0, 20.0, 20.0),
            Some("000001.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            2,
            2,
            PdfPageBox::new(0.0, 0.0, 25.0, 25.0),
            Some("000002.000002".to_string()),
        ),
    ]);

    assert_eq!(
        chunks
            .iter()
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|region| (region.page_index, region.region_index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![vec![(1, 2)], vec![(1, 1)], vec![(2, 1), (2, 2)]]
    );
}
