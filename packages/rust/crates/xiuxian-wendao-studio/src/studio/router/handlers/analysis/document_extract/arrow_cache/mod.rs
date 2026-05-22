//! Arrow IPC cache facade for document extraction results.

mod batches;
mod io;
mod mirror;
mod names;
mod schema;

#[cfg(all(test, feature = "document-extract-audio-shards"))]
pub(super) use batches::build_audio_transcript_resource_batch;
#[cfg(feature = "document-extract-audio-shards")]
pub(super) use batches::build_audio_transcript_with_org_resource_batch;
pub(super) use batches::{
    build_error_resource_batch, build_job_resource_batch, build_native_text_resource_batch,
    build_status_batch,
};
pub(super) use io::{read_arrow_file, write_arrow_file};
#[cfg(feature = "document-extract-audio-shards")]
pub(super) use mirror::mark_document_extract_cache_complete;
#[cfg(feature = "document-extract-pdf-source-range")]
pub(super) use mirror::rewrite_document_extract_resource_paths;
pub(super) use mirror::{
    mirror_artifact_to_output, mirror_document_extract_cache, read_cached_document_batches,
};
pub(super) use names::DOCUMENT_RESOURCE_ARROW_CACHE_NAME;
