//! Compatibility reexports for Julia integration public DTO carriers.
//!
//! Runtime carrier adapters live in `xiuxian-julia-runtime`; inert fact
//! catalogs live in `xiuxian-polyglot-orchestrator`.

pub use xiuxian_julia_runtime::wendao::{
    JuliaContractEnabled, JuliaContractId, JuliaContractKind, JuliaContractMode, JuliaContractPath,
    JuliaContractReason, JuliaContractRoute, JuliaContractSchemaVersion, JuliaContractSecondsU64,
    JuliaContractState, JuliaContractTimestampMsI64, JuliaContractTransport, JuliaContractUrl,
};
