//! Native context, error, and signal contracts.

mod context;
mod error;
mod signal;
mod signal_registry;

pub use context::ZhenfaContext;
pub use error::ZhenfaError;
pub use signal::ZhenfaSignal;
pub use signal_registry::{
    BroadcastResult, ExternalSignal, ObservationSignalInput, SignalRegistry, SignalRegistryExt,
};
