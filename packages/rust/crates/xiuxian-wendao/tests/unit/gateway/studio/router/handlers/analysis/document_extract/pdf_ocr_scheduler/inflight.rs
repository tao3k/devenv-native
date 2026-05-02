use super::{InFlightShardRegistry, InFlightShardReservation, PdfOcrShardResult};
use xiuxian_wendao_attachments::pdf::ocr::{PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput};

#[tokio::test]
async fn same_key_gets_single_owner_and_waiting_result() -> Result<(), String> {
    let registry = InFlightShardRegistry::default();
    let owner = registry.reserve("same-shard".to_string());
    let waiter = registry.reserve("same-shard".to_string());
    assert_eq!(registry.len(), 1);

    let InFlightShardReservation::Owner { key, entry } = owner else {
        return Err("first reservation must own shard".to_string());
    };
    let InFlightShardReservation::Waiter {
        entry: waiter_entry,
    } = waiter
    else {
        return Err("second reservation must wait for shard".to_string());
    };
    let waiter_task = tokio::spawn(async move { waiter_entry.wait().await });
    let input = sample_ocr_input();
    registry.publish(
        key.as_str(),
        &entry,
        Ok(PdfOcrShardResult::succeeded(
            &input,
            "text".to_string(),
            1.0,
        )),
    );

    let completed_result = waiter_task.await.map_err(|error| error.to_string())??;

    assert_eq!(completed_result.text.as_deref(), Some("text"));
    assert_eq!(registry.len(), 0);
    Ok(())
}

#[test]
fn published_key_can_be_reserved_again() -> Result<(), String> {
    let registry = InFlightShardRegistry::default();
    let InFlightShardReservation::Owner { key, entry } = registry.reserve("same-shard".to_string())
    else {
        return Err("first reservation must own shard".to_string());
    };
    let input = sample_ocr_input();
    registry.publish(
        key.as_str(),
        &entry,
        Ok(PdfOcrShardResult::succeeded(
            &input,
            "text".to_string(),
            1.0,
        )),
    );

    let reservation = registry.reserve("same-shard".to_string());

    assert!(matches!(
        reservation,
        InFlightShardReservation::Owner { .. }
    ));
    Ok(())
}

fn sample_ocr_input() -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "hash".to_string(),
        page_index: 0,
        image_path: "/tmp/page.png".to_string(),
        image_mime_type: "image/png".to_string(),
        raster_sha256: "raster".to_string(),
        render_profile: "source-pdf-page-range-v1".to_string(),
        ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
        ocr_engine: "docling".to_string(),
        preferred_languages: vec!["en".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 0,
        raster_height_px: 0,
        render_dpi: 0,
        rotation_degrees: 0,
        crop_left: 0.0,
        crop_bottom: 0.0,
        crop_right: 100.0,
        crop_top: 100.0,
        point_to_pixel_scale_x: 1.0,
        point_to_pixel_scale_y: 1.0,
        shard_element_id: "page-0".to_string(),
        shard_type: "page".to_string(),
        region_index: 0,
        parent_shard_element_id: String::new(),
        reading_order_key: "000000.000000".to_string(),
        source_page_pixel_left: 0,
        source_page_pixel_top: 0,
        source_page_pixel_right: 0,
        source_page_pixel_bottom: 0,
    }
}
