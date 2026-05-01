use super::super::capacity::{OcrCapacityController, OcrSchedulerLane};
use super::*;

#[test]
fn metrics_snapshot_reports_cache_lanes_and_percentiles() {
    let metrics = PdfOcrSchedulerMetrics::default();
    metrics.record_cache_resolution(3, 2);
    metrics.record_live_request(OcrSchedulerLane::SourcePdfPageRange, 4);
    metrics.record_live_request(OcrSchedulerLane::RenderedRegion, 2);
    metrics.record_queue_wait(Duration::from_millis(10));
    metrics.record_queue_wait(Duration::from_millis(40));
    metrics.record_ocr_latency(Duration::from_millis(100));
    metrics.record_ocr_latency(Duration::from_millis(300));
    let capacity = OcrCapacityController::new_with_current_budget(8, 3).snapshot();

    let snapshot = metrics.snapshot(&capacity, 6, 1);

    assert_eq!(snapshot.max_worker_bound, 8);
    assert_eq!(snapshot.current_worker_budget, 3);
    assert_eq!(snapshot.in_process_workers, 2);
    assert_eq!(snapshot.in_flight_shards, 1);
    assert_eq!(snapshot.cache_hits, 3);
    assert_eq!(snapshot.cache_misses, 2);
    assert_eq!(snapshot.live_requests, 2);
    assert_eq!(snapshot.source_pdf_page_range_shards, 4);
    assert_eq!(snapshot.rendered_region_shards, 2);
    assert_eq!(snapshot.queue_wait_p50_ms, Some(40));
    assert_eq!(snapshot.ocr_latency_p95_ms, Some(300));
}
