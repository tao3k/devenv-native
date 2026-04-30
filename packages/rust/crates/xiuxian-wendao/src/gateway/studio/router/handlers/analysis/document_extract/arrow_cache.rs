//! Arrow IPC cache facade for document extraction results.

mod batches;
mod io;
mod mirror;
mod schema;

pub(super) use batches::{
    build_error_resource_batch, build_job_resource_batch, build_status_batch,
};
pub(super) use io::{read_arrow_file, write_arrow_file};
pub(super) use mirror::{mirror_artifact_to_output, read_cached_document_batches};

pub(super) const DOCUMENT_RESOURCE_ARROW_CACHE_NAME: &str = "_resources.arrow";
