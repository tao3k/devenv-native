//! Core runtime for background job queue execution.

mod metrics;
mod runtime;
mod state;

pub use state::JobManager;
