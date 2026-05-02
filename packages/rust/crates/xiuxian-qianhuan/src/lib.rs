//! System prompt injection window based on XML Q&A blocks.
//!
//! Contract:
//! - Root tag: `<system_prompt_injection>`
//! - Entry tag: `<qa><q>...</q><a>...</a><source>...</source></qa>`
//! - `<source>` is optional.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

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

pub use config::InjectionWindowConfig;
pub use contracts::{
    InjectionMode, InjectionOrderStrategy, InjectionPolicy, InjectionSnapshot, PromptContextBlock,
    PromptContextCategory, PromptContextSource, RoleMixProfile, RoleMixRole,
};
pub use entry::QaEntry;
pub use error::InjectionError;
pub use hot_reload::{
    HotReloadDriver, HotReloadOutcome, HotReloadRuntime, HotReloadStatus, HotReloadTarget,
    HotReloadTrigger, HotReloadVersionBackend, InMemoryHotReloadVersionBackend,
    ValkeyHotReloadVersionBackend, resolve_hot_reload_watch_extensions,
    resolve_hot_reload_watch_patterns,
};
pub use interface::ManifestationInterface;
pub use manifestation::{
    EmbeddedManifestationTemplateCatalog, ManifestationManager, ManifestationRenderRequest,
    ManifestationRuntimeContext, ManifestationTemplateTarget, MemoryTemplateRecord,
};
pub use orchestrator::{InjectionLayer, ThousandFacesOrchestrator};
pub use persona::{MemoryPersonaRecord, PersonaProfile, PersonaProvider, PersonaRegistry};
pub use transmuter::{MockTransmuter, ToneTransmuter};
pub use window::SystemPromptInjectionWindow;
pub use xml::SYSTEM_PROMPT_INJECTION_TAG;
#[cfg(feature = "zhenfa-router")]
pub use zhenfa_router::{QianhuanReloadTool, QianhuanRenderTool, QianhuanZhenfaRouter};
