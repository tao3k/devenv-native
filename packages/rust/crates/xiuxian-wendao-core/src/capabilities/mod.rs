//! Capability contracts that describe supported Wendao runtime surfaces.

mod binding;
mod selector;
mod version;

pub use binding::PluginCapabilityBinding;
pub use selector::PluginProviderSelector;
pub use version::ContractVersion;
