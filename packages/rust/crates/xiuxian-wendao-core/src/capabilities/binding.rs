//! Capability binding records that connect tools to runtime contracts.

use serde::{Deserialize, Serialize};

use super::{ContractVersion, PluginProviderSelector};
use crate::artifacts::PluginLaunchSpec;
use crate::transport::{PluginTransportEndpoint, PluginTransportKind};

/// Generic runtime binding between a capability and one concrete provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCapabilityBinding {
    /// Capability id and selected provider id.
    pub selector: PluginProviderSelector,
    /// Runtime endpoint used to invoke the provider.
    pub endpoint: PluginTransportEndpoint,
    /// Optional launch specification used for managed providers.
    pub launch: Option<PluginLaunchSpec>,
    /// Data-plane transport used by this binding.
    pub transport: PluginTransportKind,
    /// Contract version negotiated for this binding.
    pub contract_version: ContractVersion,
}
