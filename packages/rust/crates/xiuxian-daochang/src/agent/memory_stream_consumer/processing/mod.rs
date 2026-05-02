//! Memory stream processing coordinates event classification, persistence writes, and recall feedback updates.

mod ack_metrics;
mod events;
mod promotion;

pub(super) use ack_metrics::ack_and_record_metrics;
pub(super) use events::process_stream_events;
pub(super) use promotion::queue_promoted_candidate;
