//! Native tool registry, context, orchestration, and signal contracts.

mod context;
mod error;
mod orchestrator;
mod registry;
mod signal;
mod signal_registry;
mod tool;

pub use context::ZhenfaContext;
pub use error::ZhenfaError;
pub use orchestrator::{
    ZhenfaAuditSink, ZhenfaDispatchEvent, ZhenfaDispatchOutcome, ZhenfaMutationGuard,
    ZhenfaMutationLock, ZhenfaOrchestrator, ZhenfaOrchestratorHooks, ZhenfaResultCache,
    ZhenfaSignalSink,
};
pub use registry::ZhenfaRegistry;
pub use signal::ZhenfaSignal;
pub use signal_registry::{
    BroadcastResult, ExternalSignal, ObservationSignalInput, SignalRegistry, SignalRegistryExt,
};
pub use tool::ZhenfaTool;
