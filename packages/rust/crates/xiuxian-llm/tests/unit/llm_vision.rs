//! Integration tests for `xiuxian_llm::llm::vision`.

use std::io::Cursor;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
use xiuxian_llm::llm::vision::{
    TextAnchor, VisibilityScrubPolicy, VisualAnchor, VisualCotInput, VisualCotMode,
    VisualRefinement, VisualUserMessageRequest, build_semantic_overlay, build_visual_cot_prompt,
    build_visual_user_message, build_visual_user_message_from_refinement,
    build_visual_user_message_with_ocr_truth, fit_dimensions, prepare_image_for_ocr_runtime,
    preprocess_image, scrub_text_anchors,
};
use xiuxian_llm::llm::{ContentPart, MessageContent};

#[test]
fn fit_dimensions_scales_by_long_edge() {
    let dimensions = fit_dimensions(4_096, 1_024, 2_048);
    assert_eq!((dimensions.width, dimensions.height), (2_048, 512));
    assert!((dimensions.scale - 0.5).abs() < f64::EPSILON);
}

#[test]
fn preprocess_image_keeps_decoded_payload_ready() -> Result<()> {
    let source_png = build_test_png(4_096, 1_024)?;
    let prepared = preprocess_image(Arc::from(source_png))?;

    assert_eq!((prepared.width, prepared.height), (2_048, 512));
    assert!(!prepared.original.is_empty());
    assert!(!prepared.resized_png.is_empty());
    Ok(())
}

#[test]
fn prepare_image_for_ocr_runtime_keeps_original_payload_for_engine_decode() -> Result<()> {
    let source_png = build_test_png(4_096, 1_024)?;
    let prepared = prepare_image_for_ocr_runtime(Arc::from(source_png))?;

    assert_eq!(prepared.mode.as_str(), "original_passthrough");
    assert_eq!((prepared.width, prepared.height), (4_096, 1_024));
    assert!((prepared.scale - 1.0).abs() < f64::EPSILON);
    assert!(Arc::ptr_eq(&prepared.original, &prepared.engine_input));
    assert_eq!(prepared.original.len(), prepared.engine_input.len());
    Ok(())
}

#[test]
fn build_semantic_overlay_formats_grounding_lines() -> Result<()> {
    let anchors = vec![
        VisualAnchor::new("submit button", 0.91, [120, 220, 160, 48]),
        VisualAnchor::new("amount input", 0.78, [80, 144, 220, 40]),
    ];

    let Some(overlay) = build_semantic_overlay(anchors.as_slice()) else {
        return Err(anyhow!("expected semantic overlay"));
    };
    assert!(overlay.contains("[vision-overlay]"));
    assert!(overlay.contains("text=\"submit button\" confidence=0.91"));
    assert!(overlay.contains("bbox=[120,220,160,48]"));
    assert!(overlay.contains("[/vision-overlay]"));
    Ok(())
}

#[test]
fn visual_cot_prompt_sorts_by_y_then_x_and_adds_strict_instruction() {
    let anchors = vec![
        TextAnchor::new("second", 0.8, [200, 100, 20, 10]),
        TextAnchor::new("first", 0.9, [50, 50, 20, 10]),
        TextAnchor::new("third", 0.7, [20, 150, 20, 10]),
        TextAnchor::new("ignored-low-confidence", 0.2, [0, 10, 10, 10]),
    ];

    let prompt = build_visual_cot_prompt(VisualCotInput {
        anchors,
        mode: VisualCotMode::Strict,
    });

    let first_idx = prompt.find("Found \"first\"").unwrap_or(usize::MAX);
    let second_idx = prompt.find("Found \"second\"").unwrap_or(usize::MAX);
    let third_idx = prompt.find("Found \"third\"").unwrap_or(usize::MAX);
    assert!(first_idx < second_idx && second_idx < third_idx);
    assert!(!prompt.contains("ignored-low-confidence"));
    assert!(prompt.contains("MUST prioritize the anchors as the ground truth"));
}

#[test]
fn visual_cot_prompt_without_anchors_is_empty() {
    let prompt = build_visual_cot_prompt(VisualCotInput {
        anchors: Vec::new(),
        mode: VisualCotMode::Assistive,
    });
    assert!(prompt.is_empty());
}

#[test]
fn visual_refinement_build_cot_prompt_prepends_user_goal() -> Result<()> {
    let source_png = build_test_png(256, 128)?;
    let prepared = preprocess_image(Arc::from(source_png))?;
    let anchors = vec![VisualAnchor::new("amount field", 0.72, [20, 40, 120, 36])];
    let refinement = VisualRefinement {
        prepared,
        anchors: anchors.clone(),
        text_anchors: anchors.iter().map(VisualAnchor::to_text_anchor).collect(),
        semantic_overlay: build_semantic_overlay(anchors.as_slice()),
        ocr_truth_markdown: None,
    };
    let prompt = refinement.build_cot_prompt(
        "Extract visible amount and verify input readiness.",
        VisualCotMode::Assistive,
    );

    assert!(prompt.contains("[VISUAL_GROUNDING_SIGNAL]"));
    assert!(prompt.contains("Found \"amount field\""));
    assert!(prompt.contains("Extract visible amount and verify input readiness."));
    Ok(())
}

#[test]
fn scrub_text_anchors_drops_noise_and_low_confidence() {
    let anchors = vec![
        TextAnchor::new("valid button", 0.95, [10, 10, 100, 30]),
        TextAnchor::new("...", 0.99, [20, 40, 10, 10]),
        TextAnchor::new("x", 0.99, [20, 60, 10, 10]),
        TextAnchor::new("low confidence", 0.10, [20, 80, 10, 10]),
    ];
    let policy = VisibilityScrubPolicy::default();
    let scrubbed = scrub_text_anchors(anchors, &policy);

    assert_eq!(scrubbed.len(), 1);
    assert_eq!(scrubbed[0].text.as_ref(), "valid button");
}

#[test]
fn build_visual_user_message_emits_high_detail_image_payload() -> Result<()> {
    let anchors = vec![TextAnchor::new("confirm", 0.88, [100, 100, 80, 24])];
    let message = build_visual_user_message(
        "Audit this payment dialog.",
        "https://example.com/screen.png",
        anchors.as_slice(),
        VisualCotMode::Assistive,
    )?;

    let Some(MessageContent::Parts(parts)) = message.content else {
        return Err(anyhow!("visual user message should use multipart content"));
    };
    assert_eq!(parts.len(), 2);
    assert!(
        matches!(&parts[0], ContentPart::Text { text } if text.contains("[VISUAL_GROUNDING_SIGNAL]"))
    );
    assert!(
        matches!(&parts[0], ContentPart::Text { text } if text.contains("Audit this payment dialog."))
    );
    assert!(
        matches!(&parts[1], ContentPart::ImageUrl { image_url } if image_url.detail.as_deref() == Some("high"))
    );
    Ok(())
}

#[test]
fn build_visual_user_message_rejects_empty_image_ref() {
    let result =
        build_visual_user_message("Inspect screenshot", "   ", &[], VisualCotMode::Assistive);
    assert!(result.is_err());
}

#[test]
fn build_visual_user_message_with_ocr_truth_prepends_truth_block() -> Result<()> {
    let message = build_visual_user_message_with_ocr_truth(VisualUserMessageRequest {
        user_message: "Summarize this invoice.",
        image_url: "https://example.com/invoice.png",
        anchors: &[],
        mode: VisualCotMode::Assistive,
        ocr_truth_markdown: Some("# Invoice\n- Total: 120.00"),
    })?;

    let Some(MessageContent::Parts(parts)) = message.content else {
        return Err(anyhow!("visual user message should use multipart content"));
    };
    assert!(
        matches!(&parts[0], ContentPart::Text { text } if text.contains("[PHYSICAL_OCR_TRUTH]:"))
    );
    assert!(matches!(&parts[0], ContentPart::Text { text } if text.contains("# Invoice")));
    assert!(
        matches!(&parts[0], ContentPart::Text { text } if text.contains("Summarize this invoice."))
    );
    Ok(())
}

#[test]
fn build_visual_user_message_from_refinement_uses_text_anchors() -> Result<()> {
    let prepared = preprocess_image(Arc::from(build_test_png(128, 64)?))?;
    let anchors = vec![VisualAnchor::new("pay now", 0.85, [12, 10, 60, 20])];
    let refinement = VisualRefinement {
        prepared,
        anchors: anchors.clone(),
        text_anchors: anchors.iter().map(VisualAnchor::to_text_anchor).collect(),
        semantic_overlay: build_semantic_overlay(anchors.as_slice()),
        ocr_truth_markdown: Some("## OCR\n- pay now".to_string()),
    };

    let message = build_visual_user_message_from_refinement(
        "Check if this button is actionable.",
        "https://example.com/frame.png",
        &refinement,
        VisualCotMode::Strict,
    )?;
    let Some(MessageContent::Parts(parts)) = message.content else {
        return Err(anyhow!("visual user message should use multipart content"));
    };
    assert!(
        matches!(&parts[0], ContentPart::Text { text } if text.contains("[PHYSICAL_OCR_TRUTH]:"))
    );
    assert!(matches!(&parts[0], ContentPart::Text { text } if text.contains("Found \"pay now\"")));
    assert!(
        matches!(&parts[0], ContentPart::Text { text } if text.contains("MUST prioritize the anchors as the ground truth"))
    );
    Ok(())
}

fn build_test_png(width: u32, height: u32) -> Result<Vec<u8>> {
    let image = ImageBuffer::from_fn(width, height, |x, y| {
        let channel = u8::try_from((x + y) % u32::from(u8::MAX)).unwrap_or(u8::MAX);
        Rgb([channel, 64, 192])
    });
    let mut cursor = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(image).write_to(&mut cursor, ImageFormat::Png)?;
    Ok(cursor.into_inner())
}
