//! Transport negotiation policy for Wendao client bindings.

use xiuxian_db_store::EngineRecordBatch;
use xiuxian_wendao_core::{capabilities::PluginCapabilityBinding, transport::PluginTransportKind};

use super::client::build_arrow_flight_transport_client_from_binding;
use super::flight::ArrowFlightTransportClient;

/// Canonical runtime transport preference order for plugin capability bindings.
pub const CANONICAL_PLUGIN_TRANSPORT_PREFERENCE_ORDER: [PluginTransportKind; 1] =
    [PluginTransportKind::ArrowFlight];

/// Negotiated runtime transport selection for one plugin client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiatedTransportSelection {
    /// Transport kind ultimately selected by the runtime.
    pub selected_transport: PluginTransportKind,
    /// Higher-preference transport kind that was skipped before selection.
    pub fallback_from: Option<PluginTransportKind>,
    /// Reason the runtime fell back from a higher-preference transport kind.
    pub fallback_reason: Option<String>,
}

/// Runtime-owned Flight transport client paired with its negotiated selection metadata.
#[derive(Clone)]
pub struct NegotiatedFlightTransportClient {
    client: ArrowFlightTransportClient,
    selection: NegotiatedTransportSelection,
}

impl NegotiatedFlightTransportClient {
    fn new(client: ArrowFlightTransportClient, selection: NegotiatedTransportSelection) -> Self {
        Self { client, selection }
    }

    /// Borrow the negotiated transport-selection metadata.
    #[must_use]
    pub fn selection(&self) -> &NegotiatedTransportSelection {
        &self.selection
    }

    /// Return the configured Arrow Flight base URL.
    #[must_use]
    pub fn flight_base_url(&self) -> &str {
        self.client.base_url()
    }

    /// Return the configured Arrow Flight route.
    #[must_use]
    pub fn flight_route(&self) -> &str {
        self.client.route()
    }

    /// Process one engine batch through the negotiated runtime transport.
    ///
    /// # Errors
    ///
    /// Returns an error when the negotiated client cannot send the request or
    /// decode the response.
    pub async fn process_batch(
        &self,
        batch: &EngineRecordBatch,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        self.client.process_batch(batch).await
    }

    /// Process multiple engine batches through the negotiated runtime transport.
    ///
    /// # Errors
    ///
    /// Returns an error when the negotiated client cannot send the request or
    /// decode the response.
    pub async fn process_batches(
        &self,
        batches: &[EngineRecordBatch],
    ) -> Result<Vec<EngineRecordBatch>, String> {
        self.client.process_batches(batches).await
    }
}

/// Negotiate the preferred runtime transport client from a set of candidate bindings.
///
/// The runtime negotiates only `ArrowFlight`.
///
/// # Errors
///
/// Returns an error when the candidate set contains configured bindings but no
/// binding can be materialized into a supported runtime transport client.
pub fn negotiate_flight_transport_client_from_bindings(
    bindings: &[PluginCapabilityBinding],
) -> Result<Option<NegotiatedFlightTransportClient>, String> {
    if bindings.is_empty() {
        return Ok(None);
    }

    let mut ordered = bindings.iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(index, binding)| (transport_preference_rank(binding.transport), *index));

    let mut fallback_from = None;
    let mut fallback_reason = None;
    let mut saw_material_failure = false;

    for (_, binding) in ordered {
        match build_runtime_flight_transport_client_from_binding(binding) {
            Ok(Some(client)) => {
                return Ok(Some(NegotiatedFlightTransportClient::new(
                    client,
                    NegotiatedTransportSelection {
                        selected_transport: binding.transport,
                        fallback_from,
                        fallback_reason,
                    },
                )));
            }
            Ok(None) => {
                fallback_from.get_or_insert(binding.transport);
                fallback_reason.get_or_insert_with(|| {
                    format!(
                        "preferred transport {:?} is unavailable because the binding has no base_url",
                        binding.transport
                    )
                });
            }
            Err(error) => {
                fallback_from.get_or_insert(binding.transport);
                fallback_reason.get_or_insert_with(|| {
                    format!(
                        "preferred transport {:?} is unavailable: {error}",
                        binding.transport
                    )
                });
                saw_material_failure = true;
            }
        }
    }

    if saw_material_failure {
        return Err(fallback_reason.unwrap_or_else(|| {
            "no supported runtime transport client could be negotiated".to_string()
        }));
    }

    Ok(None)
}

fn build_runtime_flight_transport_client_from_binding(
    binding: &PluginCapabilityBinding,
) -> Result<Option<ArrowFlightTransportClient>, String> {
    build_arrow_flight_transport_client_from_binding(binding)
}

const fn transport_preference_rank(kind: PluginTransportKind) -> usize {
    match kind {
        PluginTransportKind::ArrowFlight => 0,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/transport/negotiation.rs"]
mod tests;
