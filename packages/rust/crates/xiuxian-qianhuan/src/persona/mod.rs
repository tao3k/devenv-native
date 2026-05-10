//! Persona model and registry for xiuxian-qianhuan.

mod loader;
mod profile;
mod registry;

pub use profile::PersonaProfile;
pub use registry::{MemoryPersonaRecord, PersonaId, PersonaProvider, PersonaRegistry};
