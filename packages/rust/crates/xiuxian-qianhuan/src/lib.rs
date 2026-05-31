//! System prompt injection window based on XML Q&A blocks.
//!
//! Contract:
//! - Root tag: `<system_prompt_injection>`
//! - Entry tag: `<qa><q>...</q><a>...</a><source>...</source></qa>`
//! - `<source>` is optional.

#[cfg(feature = "artifact-cache")]
mod artifacts;
/// Synapse-Audit calibration primitives for adversarial alignment checks.
pub mod calibration;
mod config;
mod contracts;
mod entry;
mod error;
/// Shared hot-reload runtime for manifestation assets.
pub mod hot_reload;
mod interface;
/// Template manifestation manager and request contracts.
pub mod manifestation;
/// Orchestration layer for multi-layer prompt assembly.
pub mod orchestrator;
/// Persona model and registry for role-mix style injection.
pub mod persona;
/// Tone transmutation traits and implementations.
pub mod transmuter;
mod window;
mod xml;
#[cfg(feature = "zhenfa-router")]
/// Native zhenfa router adapters for qianhuan manifestation workflows.
pub mod zhenfa_router;

#[cfg(feature = "artifact-cache")]
pub use artifacts::{
    PromptContextPackIdentity, PromptContextPackReadThrough, fetch_through_injection_snapshot_pack,
    fetch_through_prompt_context_pack, prompt_context_pack_bytes, prompt_context_pack_key,
    read_through_injection_snapshot_pack, read_through_prompt_context_pack,
};
pub use config::InjectionWindowConfig;
pub use contracts::{
    InjectionMode, InjectionOrderStrategy, InjectionPolicy, InjectionSessionId, InjectionSnapshot,
    InjectionSnapshotId, InjectionSnapshotInput, InjectionTurnId, PromptContextBlock,
    PromptContextBlockId, PromptContextBlockInput, PromptContextCategory, PromptContextSource,
    PromptSessionScope, RoleMixProfile, RoleMixRole,
};
pub use entry::QaEntry;
pub use error::InjectionError;
pub use hot_reload::{
    HotReloadDriver, HotReloadOutcome, HotReloadRuntime, HotReloadStatus, HotReloadTarget,
    HotReloadTargetId, HotReloadTrigger, HotReloadVersionBackend, InMemoryHotReloadVersionBackend,
    ValkeyHotReloadVersionBackend, resolve_hot_reload_watch_extensions,
    resolve_hot_reload_watch_patterns,
};
pub use interface::ManifestationInterface;
pub use manifestation::{
    EmbeddedManifestationTemplateCatalog, ManifestationManager, ManifestationRenderRequest,
    ManifestationRuntimeContext, ManifestationTemplateTarget, MemoryTemplateRecord,
};
pub use orchestrator::{InjectionLayer, ThousandFacesOrchestrator};
pub use persona::{
    MemoryPersonaRecord, PersonaId, PersonaProfile, PersonaProvider, PersonaRegistry,
};
pub use transmuter::{MockTransmuter, ToneTransmuter};
pub use window::SystemPromptInjectionWindow;
pub use xml::SYSTEM_PROMPT_INJECTION_TAG;
#[cfg(feature = "zhenfa-router")]
pub use zhenfa_router::{QianhuanReloadTool, QianhuanRenderTool, QianhuanZhenfaRouter};
