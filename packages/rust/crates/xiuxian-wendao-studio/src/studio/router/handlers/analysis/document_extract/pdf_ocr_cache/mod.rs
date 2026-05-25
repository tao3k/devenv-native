//! OCR shard Arrow cache facade.

mod cache;
mod key;
mod policy;
mod prune;
mod resolution;
mod types;

pub(super) use key::ocr_shard_cache_key;
pub(super) use types::PdfOcrShardCache;

#[cfg(test)]
pub(super) use key::ocr_shard_artifact_key;
#[cfg(test)]
use std::time::Duration;
#[cfg(test)]
pub(super) use types::PdfOcrShardCachePolicy;
#[cfg(test)]
use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_cache.rs"]
mod tests;
