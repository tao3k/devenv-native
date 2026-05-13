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
    sorted_regions
        .chunk_by(|left, right| left.page_index == right.page_index)
        .map(<[PdfPageRegionRenderRequest]>::to_vec)
        .collect()
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
#[path = "../../../tests/unit/pdf/render/regions.rs"]
mod tests;
