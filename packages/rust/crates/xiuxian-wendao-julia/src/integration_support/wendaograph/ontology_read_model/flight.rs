//! Flight binding and runtime roundtrip helpers for the `WendaoGraph` ontology read-model bridge.

use arrow_flight::FlightDescriptor;
use xiuxian_wendao_core::PluginProviderSelector;
use xiuxian_wendao_core::capabilities::{ContractVersion, PluginCapabilityBinding};
use xiuxian_wendao_core::ids::{CapabilityId, PluginId};
use xiuxian_wendao_core::transport::{PluginTransportEndpoint, PluginTransportKind};
use xiuxian_wendao_runtime::transport::negotiate_flight_transport_client_from_bindings;

use super::constants::{
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
};
use super::ipc::{
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
};
use super::types::{
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    WendaoGraphOntologyReadModelQualityRequestBatches,
    WendaoGraphOntologyReadModelQualityRoundtrip,
    WendaoGraphOntologyReadModelQualityRoundtripError,
};

/// Build the Flight descriptor for the `WendaoGraph` ontology quality service.
#[must_use]
pub fn build_wendaograph_ontology_read_model_quality_flight_descriptor() -> FlightDescriptor {
    FlightDescriptor::new_path(
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH
            .into_iter()
            .map(str::to_owned)
            .collect(),
    )
}

/// Build the canonical provider selector for the `WendaoGraph` ontology quality service.
#[must_use]
pub fn wendaograph_ontology_read_model_quality_provider_selector() -> PluginProviderSelector {
    PluginProviderSelector {
        capability_id: CapabilityId(
            WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID.to_owned(),
        ),
        provider: PluginId(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID.to_owned()),
    }
}

/// Build one runtime-negotiable Arrow Flight binding for ontology quality scoring.
///
/// # Errors
///
/// Returns an error when the Flight base URL is blank.
pub fn build_wendaograph_ontology_read_model_quality_flight_binding(
    options: WendaoGraphOntologyReadModelQualityFlightBindingOptions,
) -> Result<PluginCapabilityBinding, String> {
    let base_url = options.base_url.trim();
    if base_url.is_empty() {
        return Err("WendaoGraph ontology quality Flight base URL must not be blank".to_string());
    }

    Ok(PluginCapabilityBinding {
        selector: wendaograph_ontology_read_model_quality_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(base_url.to_owned()),
            route: Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE.to_owned()),
            health_route: options.health_route,
            timeout_secs: options.timeout_secs,
            max_in_flight_requests: options.max_in_flight_requests,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(
            WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION.to_owned(),
        ),
    })
}

/// Run one ontology quality exchange through the shared runtime Flight transport.
///
/// # Errors
///
/// Returns [`WendaoGraphOntologyReadModelQualityRoundtripError`] when request
/// packaging, transport negotiation, or the Flight exchange fails.
pub async fn roundtrip_wendaograph_ontology_read_model_quality_with_binding(
    binding: &PluginCapabilityBinding,
    batches: &WendaoGraphOntologyReadModelQualityRequestBatches,
) -> Result<
    Option<WendaoGraphOntologyReadModelQualityRoundtrip>,
    WendaoGraphOntologyReadModelQualityRoundtripError,
> {
    let request =
        build_wendaograph_ontology_read_model_quality_arrow_request(batches).map_err(|error| {
            WendaoGraphOntologyReadModelQualityRoundtripError {
                selection: None,
                error,
            }
        })?;
    let request_batch = build_wendaograph_ontology_read_model_quality_flight_request_batch(
        &request,
    )
    .map_err(|error| WendaoGraphOntologyReadModelQualityRoundtripError {
        selection: None,
        error,
    })?;
    let Some(transport) = negotiate_flight_transport_client_from_bindings(std::slice::from_ref(
        binding,
    ))
    .map_err(|error| WendaoGraphOntologyReadModelQualityRoundtripError {
        selection: None,
        error,
    })?
    else {
        return Ok(None);
    };

    let selection = transport.selection().clone();
    let response_batches = transport
        .process_batch(&request_batch)
        .await
        .map_err(|error| WendaoGraphOntologyReadModelQualityRoundtripError {
            selection: Some(selection.clone()),
            error,
        })?;

    Ok(Some(WendaoGraphOntologyReadModelQualityRoundtrip {
        selection,
        response_batches,
    }))
}
