const LARGE_IMAGE_BYTES_THRESHOLD: u64 = 20 * 1024 * 1024;
const LARGE_IMAGE_PIXEL_THRESHOLD: u64 = 25_000_000;

pub(super) fn image_routing_decision(
    file_size_bytes: u64,
    pixel_count: Option<u64>,
) -> (&'static str, &'static str) {
    if pixel_count.is_none() {
        return (
            "docling_passthrough",
            "Rust recognized the image format but did not prove dimensions from the header",
        );
    }
    if file_size_bytes >= LARGE_IMAGE_BYTES_THRESHOLD
        || pixel_count.is_some_and(|pixels| pixels >= LARGE_IMAGE_PIXEL_THRESHOLD)
    {
        return (
            "oversized_image_preflight_candidate",
            "Rust can preflight size before a future crop or tile OCR strategy",
        );
    }
    (
        "image_ocr_cache_candidate",
        "Rust can key future whole-image OCR cache and preserve Docling as OCR authority",
    )
}
