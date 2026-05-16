//! Audio document-extract route backed by Rust shard planning and Arrow Flight.

mod config;
mod plan;
mod response;
mod route;
mod speech;

#[cfg(test)]
pub(super) use config::{audio_worker_budget_with_lookup, document_extract_audio_config};
#[cfg(test)]
pub(super) use plan::{build_full_coverage_audio_plan, parse_ffprobe_duration_ms};
#[cfg(test)]
pub(super) use response::build_audio_transcript_batch;
#[cfg(test)]
pub(super) use speech::recovery_speech_window_input_from_config;
