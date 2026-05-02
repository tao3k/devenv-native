//! Chain-of-thought redaction for vision OCR traces.

use super::anchor::TextAnchor;
use super::scrub::{VisibilityScrubPolicy, scrub_text_anchors};

/// Visual `CoT` execution modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualCotMode {
    /// Provide anchors as helpful context.
    Assistive,
    /// Mandate that anchors are prioritized over model intuition when conflicts occur.
    Strict,
}

/// Input for building a visual grounding prompt.
#[derive(Debug, Clone, PartialEq)]
pub struct VisualCotInput {
    /// Text anchors extracted locally (OCR / detector side).
    pub anchors: Vec<TextAnchor>,
    /// Grounding policy mode.
    pub mode: VisualCotMode,
}

/// Builds a structured prompt paragraph that grounds LLM reasoning in local anchors.
#[must_use]
pub fn build_visual_cot_prompt(input: VisualCotInput) -> String {
    if input.anchors.is_empty() {
        return String::new();
    }

    let mut prompt = String::from(
        "\n[VISUAL_GROUNDING_SIGNAL]: Local high-precision OCR has detected the following anchors.\n\
YOU MUST use these coordinates to verify your spatial reasoning and element descriptions.\n\
--------------------------------------------------\n",
    );

    // Scrub noisy anchors first, then sort by Y/X to follow reading flow.
    let mut anchors = scrub_text_anchors(input.anchors, &VisibilityScrubPolicy::default());
    if anchors.is_empty() {
        return String::new();
    }
    anchors.sort_by(|left, right| {
        left.bbox[1]
            .cmp(&right.bbox[1])
            .then(left.bbox[0].cmp(&right.bbox[0]))
    });

    for anchor in anchors {
        push_anchor_line(&mut prompt, &anchor);
    }

    prompt.push_str("--------------------------------------------------\n");

    if matches!(input.mode, VisualCotMode::Strict) {
        prompt.push_str(
            "INSTRUCTION: If your visual interpretation contradicts these anchors, you MUST prioritize the anchors as the ground truth.\n",
        );
    }

    prompt
}

fn push_anchor_line(prompt: &mut String, anchor: &TextAnchor) {
    prompt.push_str("- Found \"");
    prompt.push_str(&sanitize_anchor_text(anchor.text.as_ref()));
    prompt.push_str("\" at [x: ");
    prompt.push_str(&anchor.bbox[0].to_string());
    prompt.push_str(", y: ");
    prompt.push_str(&anchor.bbox[1].to_string());
    prompt.push_str(", w: ");
    prompt.push_str(&anchor.bbox[2].to_string());
    prompt.push_str(", h: ");
    prompt.push_str(&anchor.bbox[3].to_string());
    prompt.push_str("]\n");
}

fn sanitize_anchor_text(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}
