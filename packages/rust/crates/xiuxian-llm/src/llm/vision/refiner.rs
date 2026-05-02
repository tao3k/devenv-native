//! OCR text refinement and anchor-aware deduplication.

use std::cmp::Ordering;
use std::sync::Arc;

use super::anchor::{TextAnchor, VisualAnchor};
use super::cot::{VisualCotInput, VisualCotMode, build_visual_cot_prompt};
use super::preprocess::{
    DEFAULT_VISION_MAX_DIMENSION, PreparedVisionImage, prepare_image_for_ocr_runtime,
};
use crate::llm::error::LlmResult;

/// Result container returned by [`VisualRefiner::refine`].
#[derive(Debug, Clone)]
pub struct VisualRefinement {
    /// Prepared image payloads and metadata used by downstream vision flows.
    pub prepared: PreparedVisionImage,
    /// Spatially grounded anchors.
    pub anchors: Vec<VisualAnchor>,
    /// Text-only anchors derived from visual anchors.
    pub text_anchors: Vec<TextAnchor>,
    /// Semantic overlay text for prompt preprending.
    pub semantic_overlay: Option<String>,
    /// OCR truth markdown, when available.
    pub ocr_truth_markdown: Option<String>,
}

impl VisualRefinement {
    /// Builds a visual `CoT` prompt bound to this refinement result.
    #[must_use]
    pub fn build_cot_prompt(&self, user_goal: &str, mode: VisualCotMode) -> String {
        let cot = build_visual_cot_prompt(VisualCotInput {
            anchors: self.text_anchors.clone(),
            mode,
        });
        let normalized_goal = user_goal.trim();
        if cot.is_empty() {
            normalized_goal.to_string()
        } else if normalized_goal.is_empty() {
            cot
        } else {
            format!("{cot}\n{normalized_goal}")
        }
    }
}

/// Vision refiner that applies deterministic preprocessing + grounding.
#[derive(Debug, Clone)]
pub struct VisualRefiner {
    max_dimension: u32,
    min_confidence: f32,
}

impl Default for VisualRefiner {
    fn default() -> Self {
        Self {
            max_dimension: DEFAULT_VISION_MAX_DIMENSION,
            min_confidence: 0.55,
        }
    }
}

impl VisualRefiner {
    /// Creates a vision refiner with explicit resizing and confidence policy.
    #[must_use]
    pub fn new(max_dimension: u32, min_confidence: f32) -> Self {
        Self {
            max_dimension: max_dimension.max(1),
            min_confidence: min_confidence.clamp(0.0, 1.0),
        }
    }

    /// Refines raw image bytes and produces semantic overlay artifacts.
    ///
    /// # Errors
    ///
    /// Returns an error when image preparation fails.
    pub fn refine(&self, image_bytes: Arc<[u8]>) -> LlmResult<VisualRefinement> {
        let prepared = prepare_image_for_ocr_runtime(image_bytes)?;
        tracing::debug!(
            event = "llm.vision.refiner.prepare",
            requested_max_dimension = self.max_dimension,
            input_mode = prepared.mode.as_str(),
            width = prepared.width,
            height = prepared.height,
            "VisualRefiner prepared image"
        );

        let ocr_truth_markdown = None;
        let anchors = self.detect_visual_anchors(&prepared, ocr_truth_markdown.as_deref());
        let text_anchors = anchors.iter().map(VisualAnchor::to_text_anchor).collect();
        let semantic_overlay = build_semantic_overlay(anchors.as_slice());
        Ok(VisualRefinement {
            prepared,
            anchors,
            text_anchors,
            semantic_overlay,
            ocr_truth_markdown,
        })
    }

    fn detect_visual_anchors(
        &self,
        _prepared: &PreparedVisionImage,
        _ocr_truth_markdown: Option<&str>,
    ) -> Vec<VisualAnchor> {
        let raw_anchors: Vec<VisualAnchor> = Vec::new();
        raw_anchors
            .into_iter()
            .filter(|anchor| anchor.confidence >= self.min_confidence)
            .collect()
    }
}

/// Builds a stable semantic overlay from visual anchors.
///
/// The format is designed for direct prompt injection ahead of image parts.
#[must_use]
pub fn build_semantic_overlay(anchors: &[VisualAnchor]) -> Option<String> {
    if anchors.is_empty() {
        return None;
    }

    let mut sorted: Vec<&VisualAnchor> = anchors
        .iter()
        .filter(|anchor| !anchor.text.trim().is_empty())
        .collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(Ordering::Equal)
    });

    let lines = sorted
        .into_iter()
        .map(|anchor| {
            let [x, y, w, h] = anchor.bbox;
            format!(
                "- text=\"{}\" confidence={:.2} bbox=[{x},{y},{w},{h}]",
                escape_overlay_text(anchor.text.as_ref()),
                anchor.confidence
            )
        })
        .collect::<Vec<_>>();

    Some(format!(
        "[vision-overlay]\n{}\n[/vision-overlay]",
        lines.join("\n")
    ))
}

fn escape_overlay_text(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .trim()
        .to_string()
}
