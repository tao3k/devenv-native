//! Background local-project symbol index coordinator for Studio.

#[path = "state/mod.rs"]
mod state;
mod types;

pub(crate) use state::{SymbolIndexCoordinator, timestamp_now};
pub(crate) use types::{SymbolIndexPhase, SymbolIndexStatus};
