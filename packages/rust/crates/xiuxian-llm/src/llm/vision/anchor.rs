//! Text anchor geometry for vision-grounded LLM outputs.

use std::sync::Arc;

/// OCR- or heuristic-extracted textual anchor used for visual grounding.
#[derive(Debug, Clone, PartialEq)]
pub struct TextAnchor {
    /// Anchor text content.
    pub text: Arc<str>,
    /// Confidence score in the range `[0.0, 1.0]`.
    pub confidence: f32,
    /// Bounding box in `[x, y, w, h]` pixels.
    pub bbox: [u32; 4],
}

impl TextAnchor {
    /// Builds a text anchor with confidence clamped to `[0.0, 1.0]`.
    #[must_use]
    pub fn new(text: impl Into<Arc<str>>, confidence: f32, bbox: [u32; 4]) -> Self {
        Self {
            text: text.into(),
            confidence: clamp_confidence(confidence),
            bbox,
        }
    }
}

/// Spatially grounded visual anchor used to build semantic overlays.
#[derive(Debug, Clone, PartialEq)]
pub struct VisualAnchor {
    /// Anchor text content.
    pub text: Arc<str>,
    /// Confidence score in the range `[0.0, 1.0]`.
    pub confidence: f32,
    /// Bounding box in `[x, y, w, h]` pixels.
    pub bbox: [u32; 4],
}

impl VisualAnchor {
    /// Builds a visual anchor with confidence clamped to `[0.0, 1.0]`.
    #[must_use]
    pub fn new(text: impl Into<Arc<str>>, confidence: f32, bbox: [u32; 4]) -> Self {
        Self {
            text: text.into(),
            confidence: clamp_confidence(confidence),
            bbox,
        }
    }

    /// Projects a visual anchor into a text-only anchor.
    #[must_use]
    pub fn to_text_anchor(&self) -> TextAnchor {
        TextAnchor::new(Arc::clone(&self.text), self.confidence, self.bbox)
    }
}

fn clamp_confidence(confidence: f32) -> f32 {
    if confidence.is_nan() {
        0.0
    } else {
        confidence.clamp(0.0, 1.0)
    }
}
