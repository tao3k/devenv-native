//! Dataset ontology Flight provider contracts for `Wendao` runtime hosts.

use arrow_array::RecordBatch;
use async_trait::async_trait;

use crate::transport::DatasetOntologyFlightManifest;

/// Runtime-owned dataset ontology materialization Flight payload.
#[derive(Debug, Clone)]
pub struct DatasetOntologyMaterializeFlightRouteResponse {
    /// Arrow batches returned by the provider.
    pub batches: Vec<RecordBatch>,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl DatasetOntologyMaterializeFlightRouteResponse {
    /// Create one dataset ontology materialization Flight payload without
    /// application metadata.
    #[must_use]
    pub fn new(batch: RecordBatch) -> Self {
        Self {
            batches: vec![batch],
            app_metadata: Vec::new(),
        }
    }

    /// Create a dataset ontology materialization Flight payload from already
    /// materialized Arrow batches.
    #[must_use]
    pub fn from_batches(batches: Vec<RecordBatch>) -> Self {
        Self {
            batches,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through
    /// `FlightInfo.app_metadata`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Runtime-owned provider contract for dataset ontology materialization Flight
/// reads.
#[async_trait]
pub trait DatasetOntologyMaterializeFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one dataset ontology materialization response from an admitted
    /// manifest.
    ///
    /// # Errors
    ///
    /// Returns an error when the manifest cannot be materialized for the
    /// current runtime host.
    async fn dataset_ontology_materialize_batch(
        &self,
        manifest: &DatasetOntologyFlightManifest,
    ) -> Result<DatasetOntologyMaterializeFlightRouteResponse, String>;
}
