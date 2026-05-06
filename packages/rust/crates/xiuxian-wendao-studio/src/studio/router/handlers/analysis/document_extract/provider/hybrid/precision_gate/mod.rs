mod coverage;
mod gate;
mod ocr;
mod resource;
mod structure;
mod types;

pub(crate) use coverage::validate_hybrid_page_coverage;
pub(crate) use coverage::validate_hybrid_shard_coverage;
pub(crate) use gate::validate_hybrid_precision_gate;
pub(crate) use ocr::{validate_ocr_results_match_inputs, validate_successful_ocr_results};
