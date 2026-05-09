//! Vision preprocessing and semantic grounding for multimodal LLM requests.

mod anchor;
mod cot;
mod message;
mod preprocess;
mod refiner;
mod scrub;

pub use anchor::{TextAnchor, VisualAnchor};
pub use cot::{VisualCotInput, VisualCotMode, build_visual_cot_prompt};
pub use message::{
    VisualUserMessageRequest, build_visual_user_message, build_visual_user_message_from_refinement,
    build_visual_user_message_with_ocr_truth,
};
pub use preprocess::{
    DEFAULT_VISION_MAX_DIMENSION, FittedDimensions, PreparedVisionImage, PreparedVisionImageMode,
    encode_png, fit_dimensions, prepare_image_for_ocr_runtime, preprocess_image,
    preprocess_image_with_max_dimension,
};
pub use refiner::{VisualRefinement, VisualRefiner, build_semantic_overlay};
pub use scrub::{VisibilityScrubPolicy, scrub_text_anchors};
