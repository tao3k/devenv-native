//! Region render request ordering helpers.

use super::types::PdfPageRegionRenderRequest;

/// Group region render requests by page while preserving deterministic reading
/// order inside each page.
#[must_use]
pub fn page_region_render_request_chunks_by_page(
    regions: &[PdfPageRegionRenderRequest],
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
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
