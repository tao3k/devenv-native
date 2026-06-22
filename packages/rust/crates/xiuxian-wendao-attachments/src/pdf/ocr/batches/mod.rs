//! OCR shard Arrow batch builders and decoders.

mod build;
mod decode;
mod resource;
mod support;

pub use build::{
    build_ocr_shard_input_batch, build_ocr_shard_inputs, build_ocr_shard_result_batch,
};
pub use decode::{
    decode_ocr_shard_input_batch, decode_ocr_shard_input_batches, decode_ocr_shard_result_batch,
    decode_ocr_shard_result_batches,
};
pub use resource::build_ocr_result_resource_batch;
