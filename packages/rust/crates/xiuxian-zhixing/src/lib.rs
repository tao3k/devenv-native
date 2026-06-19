//! xiuxian-zhixing - The 'Unity of Knowledge and Action' logic layer.

/// Action compiler primitives for knowledge-action synthesis.
pub mod action_compiler;
/// Agenda domain models and task lifecycle logic.
pub mod agenda;
/// Alchemy-related processors and orchestration primitives.
pub mod alchemist;
/// Runtime configuration records for Zhixing-Heyi.
pub mod config;
/// Shared error types and crate-level `Result` alias.
pub mod error;
/// Core "Knowledge and Action Unity" orchestration.
pub mod heyi;
/// Secure action-selection interface contracts.
pub mod interface;
/// Journal domain model and parsing.
pub mod journal;
mod resources;
/// Storage backends for journals and agendas.
pub mod storage;

pub use action_compiler::ActionCompiler;
pub use agenda::AgendaEntry;
pub use config::ZhixingConfig;
pub use error::{Error, Result};
pub use heyi::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED,
    ManifestationInterface, PersonaProfile, ReminderQueueSettings, ReminderQueueStore,
    ReminderQueueTask, ReminderSignal, ZhixingHeyi, ZhixingHeyiInit,
};
pub use interface::{SecureAction, ZhixingLlmInterface};
pub use journal::JournalEntry;
pub use resources::RESOURCES;
