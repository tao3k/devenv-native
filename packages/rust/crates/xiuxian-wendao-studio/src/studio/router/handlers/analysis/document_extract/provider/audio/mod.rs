//! Audio document-extract route backed by Rust shard planning and Arrow Flight.

mod capacity;
mod config;
mod plan;
mod response;
mod route;
mod speech;

pub(crate) use capacity::AudioShardCapacityController;
#[cfg(test)]
pub(crate) use config::{
    AudioDocumentExtractConfig, audio_worker_budget_with_lookup, document_extract_audio_config,
};
#[cfg(test)]
pub(super) use plan::{build_full_coverage_audio_plan, parse_ffprobe_duration_ms};
#[cfg(test)]
pub(super) use response::{build_audio_transcript_batch, build_audio_transcript_with_org_batch};
#[cfg(test)]
pub(super) use route::audio_recovery_selection_options_for_plan;
#[cfg(test)]
pub(super) use speech::{
    base_speech_window_plan_from_config, recovery_speech_window_input_from_config,
};
