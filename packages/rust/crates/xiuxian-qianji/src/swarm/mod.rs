//! Swarm orchestration for multi-agent concurrent execution.
//! Start in `api`; the other modules are private owners and helpers.

#[path = "../swarm_api.rs"]
mod api;

#[path = "../swarm_discovery_model.rs"]
mod discovery_model;
#[path = "../swarm_discovery_parse.rs"]
mod discovery_parse;
#[path = "../swarm_discovery_registry.rs"]
mod discovery_registry;
#[path = "../swarm_discovery_util.rs"]
mod discovery_util;
mod engine;
#[path = "../swarm_engine_orchestrator.rs"]
mod engine_orchestrator;
#[path = "../swarm_engine_types.rs"]
mod engine_types;
#[path = "../swarm_possession_bus.rs"]
mod possession_bus;
#[path = "../swarm_possession_error_map.rs"]
mod possession_error_map;
#[path = "../swarm_possession_model.rs"]
mod possession_model;
#[path = "possession/util.rs"]
mod possession_util;

pub use self::api::{
    ClusterNodeIdentity, ClusterNodeRecord, GlobalSwarmRegistry, RemoteNodeRequest,
    RemoteNodeResponse, RemotePossessionBus, SwarmAgentConfig, SwarmAgentReport, SwarmEngine,
    SwarmExecutionOptions, SwarmExecutionReport, map_execution_error_to_response,
};
