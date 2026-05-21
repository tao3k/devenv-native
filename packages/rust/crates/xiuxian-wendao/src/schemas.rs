//! JSON schema registry for `LinkGraph` and `Wendao` protocol blocks.

/// Canonical schema for `LinkGraph` retrieval plans.
pub const LINK_GRAPH_RETRIEVAL_PLAN_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.retrieval_plan.v1.schema.json");

/// Canonical schema for `LinkGraph` search options.
pub const LINK_GRAPH_SEARCH_OPTIONS_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.search_options.v1.schema.json");

/// Canonical schema for `LinkGraph` search options v2.
pub const LINK_GRAPH_SEARCH_OPTIONS_V2: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.search_options.v2.schema.json");

/// Canonical schema for `LinkGraph` records.
pub const LINK_GRAPH_RECORD_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.record.v1.schema.json");

/// Canonical schema for `LinkGraph` suggested-link proposals.
pub const LINK_GRAPH_SUGGESTED_LINK_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.suggested_link.v1.schema.json");

/// Canonical schema for `LinkGraph` suggested-link decisions.
pub const LINK_GRAPH_SUGGESTED_LINK_DECISION_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.suggested_link_decision.v1.schema.json");

/// Canonical schema for `LinkGraph` stats cache.
pub const LINK_GRAPH_STATS_CACHE_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.stats.cache.v1.schema.json");

/// Canonical schema for `LinkGraph` quantum context snapshots.
pub const LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.quantum_context_snapshot.v1.schema.json");

/// Canonical schema for `LinkGraph` cache snapshots.
pub const LINK_GRAPH_CACHE_SNAPSHOT_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.cache_snapshot.v1.schema.json");

/// Canonical schema for legacy `LinkGraph` valkey cache snapshots.
pub const LINK_GRAPH_VALKEY_CACHE_SNAPSHOT_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.valkey_cache_snapshot.v1.schema.json");

/// Canonical schema for `HMAS` task protocol blocks.
pub const HMAS_TASK_V1: &str = include_str!("../resources/xiuxian_wendao.hmas.task.v1.schema.json");

/// Canonical schema for `HMAS` conclusion protocol blocks.
pub const HMAS_CONCLUSION_V1: &str =
    include_str!("../resources/xiuxian_wendao.hmas.conclusion.v1.schema.json");

/// Canonical schema for `HMAS` digital thread protocol blocks.
pub const HMAS_DIGITAL_THREAD_V1: &str =
    include_str!("../resources/xiuxian_wendao.hmas.digital_thread.v1.schema.json");

/// Canonical schema for `HMAS` evidence protocol blocks.
pub const HMAS_EVIDENCE_V1: &str =
    include_str!("../resources/xiuxian_wendao.hmas.evidence.v1.schema.json");

// --- agent runtime schemas ---
/// Canonical schema for agent route traces.
pub const AGENT_ROUTE_TRACE_V1: &str =
    include_str!("../resources/agent/xiuxian.runtime.route_trace.v1.schema.json");
/// Canonical schema for agent server info.
pub const AGENT_SERVER_INFO_V1: &str =
    include_str!("../resources/agent/xiuxian.runtime.server_info.v1.schema.json");
/// Canonical schema for agent session closure.
pub const AGENT_SESSION_CLOSED_V1: &str =
    include_str!("../resources/agent/xiuxian.runtime.session_closed.v1.schema.json");
/// Canonical schema for router route tests.
pub const ROUTER_ROUTE_TEST_V1: &str =
    include_str!("../resources/agent/xiuxian.router.route_test.v1.schema.json");
/// Canonical schema for router search operations.
pub const ROUTER_ROUTING_SEARCH_V1: &str =
    include_str!("../resources/agent/xiuxian.router.routing_search.v1.schema.json");
/// Canonical schema for router search configuration.
pub const ROUTER_SEARCH_CONFIG_V1: &str =
    include_str!("../resources/agent/xiuxian.router.search_config.v1.schema.json");
/// Canonical schema for agent discovery matches.
pub const DISCOVER_MATCH_V1: &str =
    include_str!("../resources/agent/xiuxian.discover.match.v1.schema.json");

// --- xiuxian-memory-engine schemas ---
/// Canonical schema for `memory-engine` gate events.
pub const MEMORY_GATE_EVENT_V1: &str =
    include_str!("../../xiuxian-memory-engine/resources/xiuxian.memory.gate_event.v1.schema.json");

// --- Wendao skill schemas ---
/// Canonical schema for skill metadata.
pub const SKILL_METADATA_V1: &str = include_str!("../resources/skill_metadata.schema.json");
/// Canonical schema for skill command indexes.
pub const SKILL_COMMAND_INDEX_V1: &str =
    include_str!("../resources/xiuxian.skill.command_index.v1.schema.json");

// --- xiuxian-runtime schemas ---
/// Canonical schema for runtime tool results.
pub const TOOL_RESULT_V1: &str =
    include_str!("../resources/xiuxian.runtime.tool_result.v1.schema.json");

/// Resolve a schema by its canonical name.
#[must_use]
pub fn get_schema(name: &str) -> Option<&'static str> {
    match name {
        "xiuxian_wendao.link_graph.record.v1" => Some(LINK_GRAPH_RECORD_V1),
        "xiuxian_wendao.link_graph.retrieval_plan.v1" => Some(LINK_GRAPH_RETRIEVAL_PLAN_V1),
        "xiuxian_wendao.link_graph.search_options.v1" => Some(LINK_GRAPH_SEARCH_OPTIONS_V1),
        "xiuxian_wendao.link_graph.search_options.v2" => Some(LINK_GRAPH_SEARCH_OPTIONS_V2),
        "xiuxian_wendao.link_graph.suggested_link.v1" => Some(LINK_GRAPH_SUGGESTED_LINK_V1),
        "xiuxian_wendao.link_graph.suggested_link_decision.v1" => {
            Some(LINK_GRAPH_SUGGESTED_LINK_DECISION_V1)
        }
        "xiuxian_wendao.link_graph.stats.cache.v1" => Some(LINK_GRAPH_STATS_CACHE_V1),
        "xiuxian_wendao.link_graph.quantum_context_snapshot.v1" => {
            Some(LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_V1)
        }
        "xiuxian_wendao.link_graph.cache_snapshot.v1" => Some(LINK_GRAPH_CACHE_SNAPSHOT_V1),
        "xiuxian_wendao.link_graph.valkey_cache_snapshot.v1" => {
            Some(LINK_GRAPH_VALKEY_CACHE_SNAPSHOT_V1)
        }
        "xiuxian_wendao.hmas.task.v1" => Some(HMAS_TASK_V1),
        "xiuxian_wendao.hmas.conclusion.v1" => Some(HMAS_CONCLUSION_V1),
        "xiuxian_wendao.hmas.digital_thread.v1" => Some(HMAS_DIGITAL_THREAD_V1),
        "xiuxian_wendao.hmas.evidence.v1" => Some(HMAS_EVIDENCE_V1),
        "xiuxian.skill.command_index.v1" => Some(SKILL_COMMAND_INDEX_V1),
        "xiuxian.runtime.tool_result.v1" => Some(TOOL_RESULT_V1),
        _ => None,
    }
}
