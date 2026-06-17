//! Runtime data contracts for `Qianji` bootcamp workflow execution.

use crate::scheduler_preflight::RuntimeWendaoMount;
use include_dir::Dir;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_wendao::link_graph::LinkGraphIndex;

/// Runtime report returned by the Qianji laboratory API.
///
/// Raw DTO boundary: bootcamp reports preserve primitive transport fields so
/// callers can serialize workflow URIs and millisecond timestamps without
/// depending on scheduler-internal newtypes.
#[derive(Debug, Clone)]
pub struct WorkflowReport {
    /// Input canonical workflow URI.
    pub flow_uri: String,
    /// Parsed manifest name.
    pub manifest_name: String,
    /// Number of nodes declared by the manifest.
    pub node_count: usize,
    /// Number of edges declared by the manifest.
    pub edge_count: usize,
    /// Whether this manifest requires LLM capability.
    pub requires_llm: bool,
    /// UNIX epoch start timestamp in milliseconds.
    pub started_at_unix_ms: u128,
    /// UNIX epoch finish timestamp in milliseconds.
    pub finished_at_unix_ms: u128,
    /// End-to-end workflow execution duration in milliseconds.
    pub duration_ms: u128,
    /// Final merged workflow context after execution.
    pub final_context: Value,
}

/// Optional runtime overrides for `run_workflow`.
#[derive(Clone)]
pub struct BootcampRunOptions {
    /// Optional project root for `LinkGraph` bootstrap.
    ///
    /// Resolution order when omitted:
    /// 1. `PRJ_ROOT` env var
    /// 2. process current working directory
    pub repo_path: Option<PathBuf>,
    /// Optional session id for checkpoint persistence.
    pub session_id: Option<String>,
    /// Optional `Valkey` URL used with `session_id`.
    pub redis_url: Option<String>,
    /// Genesis rules for default orchestrator construction.
    pub genesis_rules: String,
    /// Optional prebuilt `LinkGraph` index.
    pub index: Option<Arc<LinkGraphIndex>>,
    /// Optional prebuilt `Qianhuan` orchestrator.
    pub orchestrator: Option<Arc<ThousandFacesOrchestrator>>,
    /// Optional prebuilt persona registry.
    pub persona_registry: Option<Arc<PersonaRegistry>>,
    /// Optional manager for distributed consensus voting.
    pub consensus_manager: Option<Arc<crate::consensus::ConsensusManager>>,
}

impl BootcampRunOptions {
    /// Creates options with safe defaults for local bootcamp execution.
    #[must_use]
    pub fn new() -> Self {
        Self {
            repo_path: None,
            session_id: None,
            redis_url: None,
            genesis_rules: "Safety Rules".to_string(),
            index: None,
            orchestrator: None,
            persona_registry: None,
            consensus_manager: None,
        }
    }
}

impl Default for BootcampRunOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// External VFS mount descriptor used by `run_scenario`.
#[derive(Debug, Clone, Copy)]
pub struct BootcampVfsMount {
    /// Semantic skill name used in URI host segment:
    /// `wendao://skills/<semantic_name>/references/...`.
    pub semantic_name: &'static str,
    /// References directory path inside `dir`, for example:
    /// `zhixing/skills/agenda-management/references`.
    pub references_dir: &'static str,
    /// Embedded directory exported by the source crate.
    pub dir: &'static Dir<'static>,
}

impl BootcampVfsMount {
    /// Creates one explicit mount descriptor.
    #[must_use]
    pub const fn new(
        semantic_name: &'static str,
        references_dir: &'static str,
        dir: &'static Dir<'static>,
    ) -> Self {
        Self {
            semantic_name,
            references_dir,
            dir,
        }
    }
}

impl From<BootcampVfsMount> for RuntimeWendaoMount {
    fn from(value: BootcampVfsMount) -> Self {
        Self {
            semantic_name: value.semantic_name,
            references_dir: value.references_dir,
            dir: value.dir,
        }
    }
}
