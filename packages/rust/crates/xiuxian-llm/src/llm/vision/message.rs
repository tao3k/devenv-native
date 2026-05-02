//! Vision prompt message construction for image content.

use crate::llm::client::{ChatMessage, ContentPart, ImageUrlContent, MessageContent, MessageRole};
use crate::llm::error::{LlmError, LlmResult};

use super::anchor::TextAnchor;
use super::cot::{VisualCotInput, VisualCotMode, build_visual_cot_prompt};
use super::refiner::VisualRefinement;

/// Builds a multimodal user message with visual `CoT` guidance and high-detail image payload.
///
/// # Errors
///
/// Returns an error when the provided image URL/reference is empty or contains
/// newline characters.
pub fn build_visual_user_message(
    user_message: &str,
    image_url: &str,
    anchors: &[TextAnchor],
    mode: VisualCotMode,
) -> LlmResult<ChatMessage> {
    build_visual_user_message_with_ocr_truth(user_message, image_url, anchors, mode, None)
}

/// Builds a multimodal user message with optional physical OCR truth injection.
///
/// # Errors
///
/// Returns an error when the provided image URL/reference is empty or contains
/// newline characters.
pub fn build_visual_user_message_with_ocr_truth(
    user_message: &str,
    image_url: &str,
    anchors: &[TextAnchor],
    mode: VisualCotMode,
    ocr_truth_markdown: Option<&str>,
) -> LlmResult<ChatMessage> {
    let normalized_image_url = image_url.trim();
    if normalized_image_url.is_empty() || normalized_image_url.contains('\n') {
        return Err(LlmError::InvalidImageReference);
    }

    let grounded_user_message =
        compose_grounded_user_text(user_message, anchors, mode, ocr_truth_markdown);

    Ok(ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Parts(vec![
            ContentPart::Text {
                text: grounded_user_message,
            },
            ContentPart::ImageUrl {
                image_url: ImageUrlContent {
                    url: normalized_image_url.to_string(),
                    detail: Some("high".to_string()),
                },
            },
        ])),
        function_call: None,
        name: None,
        tool_call_id: None,
        tool_calls: None,
        thinking: None,
    })
}

/// Builds a multimodal user message directly from a visual refinement output.
///
/// # Errors
///
/// Returns an error when the provided image URL/reference is empty or contains
/// newline characters.
pub fn build_visual_user_message_from_refinement(
    user_message: &str,
    image_url: &str,
    refinement: &VisualRefinement,
    mode: VisualCotMode,
) -> LlmResult<ChatMessage> {
    build_visual_user_message_with_ocr_truth(
        user_message,
        image_url,
        refinement.text_anchors.as_slice(),
        mode,
        refinement.ocr_truth_markdown.as_deref(),
    )
}

fn compose_grounded_user_text(
    user_message: &str,
    anchors: &[TextAnchor],
    mode: VisualCotMode,
    ocr_truth_markdown: Option<&str>,
) -> String {
    let mut blocks = Vec::new();

    if let Some(ocr_truth) = ocr_truth_markdown
        .map(str::trim)
        .filter(|ocr_truth| !ocr_truth.is_empty())
    {
        blocks.push(format!(
            "[PHYSICAL_OCR_TRUTH]: The following is a high-fidelity Markdown reconstruction of the image.\n\n{ocr_truth}\n\n[INSTRUCTION]: Use this truth to answer the user query."
        ));
    }

    let cot_text = build_visual_cot_prompt(VisualCotInput {
        anchors: anchors.to_vec(),
        mode,
    });
    if !cot_text.is_empty() {
        blocks.push(cot_text);
    }

    let normalized_user_message = user_message.trim();
    if !normalized_user_message.is_empty() {
        blocks.push(normalized_user_message.to_string());
    }

    blocks.join("\n\n")
}
