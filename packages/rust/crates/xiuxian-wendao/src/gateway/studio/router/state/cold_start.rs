use std::collections::BTreeMap;

use chrono::DateTime;
use serde::Serialize;

use crate::gateway::studio::router::state::types::StudioState;
use crate::gateway::studio::symbol_index::timestamp_now;
use crate::search::{SearchBuildRepeatWorkTelemetry, SearchCorpusKind, SearchPlaneStatusSnapshot};

const COLD_START_WINDOW_MS: u64 = 60_000;
const LOCAL_COLD_START_CORPORA: [SearchCorpusKind; 4] = [
    SearchCorpusKind::KnowledgeSection,
    SearchCorpusKind::Attachment,
    SearchCorpusKind::LocalSymbol,
    SearchCorpusKind::ReferenceOccurrence,
];

/// One process-local cold-start event observed by the Studio gateway.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioSearchColdStartEvent {
    /// RFC3339 wall-clock timestamp captured when the event was first observed.
    pub recorded_at: String,
    /// Milliseconds elapsed since the current Studio process started.
    pub elapsed_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional stable source label that first observed the event.
    pub source: Option<String>,
}

/// Process-local cold-start telemetry for one local search corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioSearchColdStartCorpusTelemetry {
    /// Stable corpus identifier.
    pub corpus: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// First time this process asked the search plane to start the corpus index.
    pub first_index_started: Option<StudioSearchColdStartEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// First time this process observed a readable active epoch for the corpus.
    pub first_ready_observed: Option<StudioSearchColdStartEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// First time a search route returned a partial response for the corpus.
    pub first_partial_search_response: Option<StudioSearchColdStartEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// First time a search route returned a ready response for the corpus.
    pub first_ready_search_response: Option<StudioSearchColdStartEvent>,
}

/// Aggregated process-local cold-start telemetry surfaced by the status endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioSearchColdStartDiagnosticsTelemetry {
    /// Repeat-work detector for local corpus build activity.
    pub repeat_work: SearchBuildRepeatWorkTelemetry,
}

/// Aggregated process-local cold-start telemetry surfaced by the status endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioSearchColdStartTelemetry {
    /// RFC3339 timestamp captured when the current Studio process started.
    pub process_started_at: String,
    /// Milliseconds elapsed since the current Studio process started.
    pub process_uptime_ms: u64,
    /// Stable cold-start observation window used by the status endpoint.
    pub cold_start_window_ms: u64,
    /// Whether the current process is still inside the cold-start window.
    pub cold_start_window_open: bool,
    /// Stable per-corpus cold-start observations for local corpora.
    pub corpora: Vec<StudioSearchColdStartCorpusTelemetry>,
    /// Structured telemetry detectors grouped under the cold-start framework.
    pub diagnostics: StudioSearchColdStartDiagnosticsTelemetry,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct StudioSearchColdStartCorpusState {
    pub(crate) index_started: Option<StudioSearchColdStartEvent>,
    pub(crate) ready_observed: Option<StudioSearchColdStartEvent>,
    pub(crate) partial_search_response: Option<StudioSearchColdStartEvent>,
    pub(crate) ready_search_response: Option<StudioSearchColdStartEvent>,
}

#[derive(Debug, Default)]
pub(crate) struct StudioSearchColdStartTelemetryState {
    corpora: BTreeMap<SearchCorpusKind, StudioSearchColdStartCorpusState>,
}

impl StudioSearchColdStartTelemetryState {
    fn corpus_mut(&mut self, corpus: SearchCorpusKind) -> &mut StudioSearchColdStartCorpusState {
        self.corpora.entry(corpus).or_default()
    }
}

impl StudioState {
    /// Returns the current process-local cold-start telemetry snapshot.
    #[must_use]
    pub fn search_cold_start_telemetry(&self) -> StudioSearchColdStartTelemetry {
        let process_uptime_ms = self.process_uptime_ms();
        let telemetry = self
            .cold_start_telemetry
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let corpora = LOCAL_COLD_START_CORPORA
            .iter()
            .copied()
            .map(|corpus| {
                let state = telemetry.corpora.get(&corpus).cloned().unwrap_or_default();
                StudioSearchColdStartCorpusTelemetry {
                    corpus: corpus.as_str().to_string(),
                    first_index_started: state.index_started,
                    first_ready_observed: state.ready_observed,
                    first_partial_search_response: state.partial_search_response,
                    first_ready_search_response: state.ready_search_response,
                }
            })
            .collect();
        StudioSearchColdStartTelemetry {
            process_started_at: self.cold_start_process_started_at.clone(),
            process_uptime_ms,
            cold_start_window_ms: COLD_START_WINDOW_MS,
            cold_start_window_open: process_uptime_ms <= COLD_START_WINDOW_MS,
            corpora,
            diagnostics: StudioSearchColdStartDiagnosticsTelemetry {
                repeat_work: self.search_plane.repeat_work_telemetry(),
            },
        }
    }

    pub(crate) fn record_local_corpus_index_started(
        &self,
        corpus: SearchCorpusKind,
        source: &'static str,
    ) {
        self.record_local_cold_start_event(corpus, source, |state, event| {
            record_first_event(&mut state.index_started, event);
        });
    }

    pub(crate) fn record_local_corpus_ready_observed(
        &self,
        corpus: SearchCorpusKind,
        source: &'static str,
    ) {
        self.record_local_cold_start_event(corpus, source, |state, event| {
            record_first_event(&mut state.ready_observed, event);
        });
    }

    pub(crate) fn record_local_corpus_ready_observed_with_recorded_at(
        &self,
        corpus: SearchCorpusKind,
        recorded_at: &str,
        source: &'static str,
    ) {
        let event = self
            .cold_start_event_from_recorded_at(recorded_at, source)
            .unwrap_or_else(|| self.cold_start_event_now(source));
        self.record_local_cold_start_event_with_value(corpus, event, |state, event| {
            record_first_event(&mut state.ready_observed, event);
        });
    }

    pub(crate) fn record_local_corpus_partial_search_response(
        &self,
        corpus: SearchCorpusKind,
        source: &'static str,
    ) {
        self.record_local_cold_start_event(corpus, source, |state, event| {
            record_first_event(&mut state.partial_search_response, event);
        });
    }

    pub(crate) fn record_local_corpus_ready_search_response(
        &self,
        corpus: SearchCorpusKind,
        source: &'static str,
    ) {
        self.record_local_cold_start_event(corpus, source, |state, event| {
            record_first_event(&mut state.ready_search_response, event);
        });
    }

    pub(crate) fn record_local_corpus_ready_observations_from_snapshot(
        &self,
        snapshot: &SearchPlaneStatusSnapshot,
        source: &'static str,
    ) {
        for status in snapshot.corpora.iter().filter(|status| {
            status.corpus == SearchCorpusKind::KnowledgeSection
                || status.corpus == SearchCorpusKind::Attachment
                || status.corpus == SearchCorpusKind::LocalSymbol
                || status.corpus == SearchCorpusKind::ReferenceOccurrence
        }) {
            if status.active_epoch.is_some() {
                if let Some(build_finished_at) = status.build_finished_at.as_deref() {
                    self.record_local_corpus_ready_observed_with_recorded_at(
                        status.corpus,
                        build_finished_at,
                        source,
                    );
                } else {
                    self.record_local_corpus_ready_observed(status.corpus, source);
                }
            }
        }
    }

    fn record_local_cold_start_event(
        &self,
        corpus: SearchCorpusKind,
        source: &'static str,
        update: impl FnOnce(&mut StudioSearchColdStartCorpusState, StudioSearchColdStartEvent),
    ) {
        let event = self.cold_start_event_now(source);
        self.record_local_cold_start_event_with_value(corpus, event, update);
    }

    fn record_local_cold_start_event_with_value(
        &self,
        corpus: SearchCorpusKind,
        event: StudioSearchColdStartEvent,
        update: impl FnOnce(&mut StudioSearchColdStartCorpusState, StudioSearchColdStartEvent),
    ) {
        let mut telemetry = self
            .cold_start_telemetry
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let state = telemetry.corpus_mut(corpus);
        update(state, event);
    }

    fn cold_start_event_now(&self, source: &'static str) -> StudioSearchColdStartEvent {
        StudioSearchColdStartEvent {
            recorded_at: timestamp_now(),
            elapsed_ms: self.process_uptime_ms(),
            source: Some(source.to_string()),
        }
    }

    fn cold_start_event_from_recorded_at(
        &self,
        recorded_at: &str,
        source: &'static str,
    ) -> Option<StudioSearchColdStartEvent> {
        let process_started_at =
            DateTime::parse_from_rfc3339(&self.cold_start_process_started_at).ok()?;
        let recorded_at = DateTime::parse_from_rfc3339(recorded_at).ok()?;
        let elapsed_ms = recorded_at
            .signed_duration_since(process_started_at)
            .num_milliseconds()
            .max(0);
        Some(StudioSearchColdStartEvent {
            recorded_at: recorded_at.to_rfc3339(),
            elapsed_ms: u64::try_from(elapsed_ms).unwrap_or(u64::MAX),
            source: Some(source.to_string()),
        })
    }

    fn process_uptime_ms(&self) -> u64 {
        self.cold_start_process_started_instant
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }
}

fn record_first_event(
    slot: &mut Option<StudioSearchColdStartEvent>,
    event: StudioSearchColdStartEvent,
) {
    if slot.is_none() {
        *slot = Some(event);
    }
}
