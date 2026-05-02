//! Discord managed job-command branch for status and summary handling.

mod status;
mod submit;
mod summary;

pub(super) use status::handle_job_status;
pub(super) use submit::handle_background_submit;
pub(super) use summary::handle_jobs_summary;
