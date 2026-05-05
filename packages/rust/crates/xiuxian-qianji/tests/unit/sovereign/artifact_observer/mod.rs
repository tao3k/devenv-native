use super::{
    ArtifactIngestionResult, ArtifactObserver, ArtifactObserverBuilder, ArtifactObserverConfig,
    NoopWendaoIngestionSink, WendaoIngestionSink,
};
use crate::telemetry::{NodeTransitionPhase, SwarmEvent};
use async_trait::async_trait;
use std::sync::Arc;
use xiuxian_wendao_core::{CognitiveTraceRecord, LinkGraphSemanticDocument};

mod builder;
mod config_and_results;
mod event_handling;
mod ingestion;
