//! Region render request ordering helpers.

use super::types::PdfPageRegionRenderRequest;

/// Group region render requests by page while preserving deterministic reading
/// order inside each page.
#[must_use]
pub fn page_region_render_request_chunks_by_page(
    regions: &[PdfPageRegionRenderRequest],
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    let sorted_regions = sorted_region_requests(regions);
    page_region_render_request_chunks_by_sorted_page(sorted_regions)
}

/// Render all region requests in one deterministic chunk.
///
/// This avoids repeatedly loading the same source PDF when the caller wants
/// one batch render before dispatching OCR work.
#[must_use]
pub fn page_region_render_request_chunks_all(
    regions: &[PdfPageRegionRenderRequest],
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    let sorted_regions = sorted_region_requests(regions);
    if sorted_regions.is_empty() {
        Vec::new()
    } else {
        vec![sorted_regions]
    }
}

fn sorted_region_requests(
    regions: &[PdfPageRegionRenderRequest],
) -> Vec<PdfPageRegionRenderRequest> {
    let mut sorted_regions = regions.to_vec();
    sorted_regions.sort_by(|left, right| {
        left.page_index
            .cmp(&right.page_index)
            .then_with(|| {
                left.effective_reading_order_key()
                    .cmp(&right.effective_reading_order_key())
            })
            .then_with(|| left.region_index.cmp(&right.region_index))
    });
    sorted_regions
}

fn page_region_render_request_chunks_by_sorted_page(
    sorted_regions: Vec<PdfPageRegionRenderRequest>,
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    let mut chunks = Vec::new();
    let mut current_page = None;
    let mut current_regions = Vec::new();
    for region in sorted_regions {
        if current_page == Some(region.page_index) {
            current_regions.push(region);
            continue;
        }
        if !current_regions.is_empty() {
            chunks.push(current_regions);
        }
        current_page = Some(region.page_index);
        current_regions = vec![region];
    }
    if !current_regions.is_empty() {
        chunks.push(current_regions);
    }
    chunks
}

/// Group region render requests by page, then dispatch pages with the largest
/// total region area first. Reading order inside each page stays deterministic.
#[must_use]
pub fn page_region_render_request_chunks_by_page_area_desc(
    regions: &[PdfPageRegionRenderRequest],
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    let mut chunks = page_region_render_request_chunks_by_page(regions);
    chunks.sort_by(|left, right| {
        page_region_chunk_area(right)
            .total_cmp(&page_region_chunk_area(left))
            .then_with(|| {
                left.first()
                    .map(|region| region.page_index)
                    .cmp(&right.first().map(|region| region.page_index))
            })
    });
    chunks
}

/// Group region render requests by page, then dispatch pages with the largest
/// single region first. Reading order inside each page stays deterministic.
#[must_use]
pub fn page_region_render_request_chunks_by_page_max_area_desc(
    regions: &[PdfPageRegionRenderRequest],
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    let mut chunks = page_region_render_request_chunks_by_page(regions);
    chunks.sort_by(|left, right| {
        page_region_chunk_max_area(right)
            .total_cmp(&page_region_chunk_max_area(left))
            .then_with(|| page_region_chunk_area(right).total_cmp(&page_region_chunk_area(left)))
            .then_with(|| {
                left.first()
                    .map(|region| region.page_index)
                    .cmp(&right.first().map(|region| region.page_index))
            })
    });
    chunks
}

/// Split region render requests into single-region chunks while preserving the
/// same deterministic reading order as page-level chunks.
#[must_use]
pub fn page_region_render_request_chunks_by_region(
    regions: &[PdfPageRegionRenderRequest],
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    sorted_region_requests(regions)
        .into_iter()
        .map(|region| vec![region])
        .collect()
}

/// Render a small single-region seed first, then render the remaining regions
/// grouped by page.
///
/// This keeps the common page-grouped tail shape while allowing hosted OCR
/// dispatch to start as soon as the cheapest region render is available.
#[must_use]
pub fn page_region_render_request_chunks_by_region_seed_page(
    regions: &[PdfPageRegionRenderRequest],
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    let sorted_regions = sorted_region_requests(regions);
    let Some((seed_index, _)) =
        sorted_regions
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                region_area(left)
                    .total_cmp(&region_area(right))
                    .then_with(|| left.page_index.cmp(&right.page_index))
                    .then_with(|| {
                        left.effective_reading_order_key()
                            .cmp(&right.effective_reading_order_key())
                    })
                    .then_with(|| left.region_index.cmp(&right.region_index))
            })
    else {
        return Vec::new();
    };

    let mut remaining_regions = sorted_regions;
    let seed = remaining_regions.remove(seed_index);
    let mut chunks = vec![vec![seed]];
    chunks.extend(page_region_render_request_chunks_by_sorted_page(
        remaining_regions,
    ));
    chunks
}

fn page_region_chunk_area(regions: &[PdfPageRegionRenderRequest]) -> f64 {
    regions.iter().map(region_area).sum()
}

fn page_region_chunk_max_area(regions: &[PdfPageRegionRenderRequest]) -> f64 {
    regions
        .iter()
        .map(region_area)
        .max_by(f64::total_cmp)
        .unwrap_or(0.0)
}

fn region_area(region: &PdfPageRegionRenderRequest) -> f64 {
    region.region_box.width_points() * region.region_box.height_points()
}

#[cfg(test)]
mod tests {
    use super::{
        PdfPageRegionRenderRequest, page_region_render_request_chunks_all,
        page_region_render_request_chunks_by_page_area_desc,
        page_region_render_request_chunks_by_page_max_area_desc,
        page_region_render_request_chunks_by_region,
        page_region_render_request_chunks_by_region_seed_page,
    };
    use crate::pdf::render::PdfPageBox;

    #[test]
    fn page_region_render_request_chunks_by_region_preserve_reading_order() {
        let chunks = page_region_render_request_chunks_by_region(&[
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

        assert_eq!(chunks.len(), 3);
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
            vec![vec![(1, 1)], vec![(2, 1)], vec![(2, 2)]]
        );
    }

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
    fn page_region_render_request_chunks_by_page_area_desc_prioritize_large_pages() {
        let chunks = page_region_render_request_chunks_by_page_area_desc(&[
            PdfPageRegionRenderRequest::new(
                2,
                1,
                PdfPageBox::new(0.0, 0.0, 10.0, 10.0),
                Some("000002.000001".to_string()),
            ),
            PdfPageRegionRenderRequest::new(
                1,
                2,
                PdfPageBox::new(0.0, 0.0, 15.0, 15.0),
                Some("000001.000002".to_string()),
            ),
            PdfPageRegionRenderRequest::new(
                1,
                1,
                PdfPageBox::new(0.0, 0.0, 15.0, 15.0),
                Some("000001.000001".to_string()),
            ),
            PdfPageRegionRenderRequest::new(
                3,
                1,
                PdfPageBox::new(0.0, 0.0, 8.0, 8.0),
                Some("000003.000001".to_string()),
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
            vec![vec![(1, 1), (1, 2)], vec![(2, 1)], vec![(3, 1)]]
        );
    }

    #[test]
    fn page_region_render_request_chunks_by_page_max_area_desc_prioritize_largest_region() {
        let chunks = page_region_render_request_chunks_by_page_max_area_desc(&[
            PdfPageRegionRenderRequest::new(
                2,
                1,
                PdfPageBox::new(0.0, 0.0, 30.0, 30.0),
                Some("000002.000001".to_string()),
            ),
            PdfPageRegionRenderRequest::new(
                1,
                2,
                PdfPageBox::new(0.0, 0.0, 25.0, 25.0),
                Some("000001.000002".to_string()),
            ),
            PdfPageRegionRenderRequest::new(
                1,
                1,
                PdfPageBox::new(0.0, 0.0, 25.0, 25.0),
                Some("000001.000001".to_string()),
            ),
            PdfPageRegionRenderRequest::new(
                3,
                1,
                PdfPageBox::new(0.0, 0.0, 8.0, 8.0),
                Some("000003.000001".to_string()),
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
            vec![vec![(2, 1)], vec![(1, 1), (1, 2)], vec![(3, 1)]]
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
}
