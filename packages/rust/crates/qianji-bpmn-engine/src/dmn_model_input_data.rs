use crate::dmn_model_document::DmnInputDataSnapshot;
use std::sync::Arc;

/// One bounded executable DMN input-data contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInputDataDefinition {
    /// Source identifier used for diagnostics.
    pub source_id: Arc<str>,
    /// Stable top-level `inputData` identifier when present.
    pub input_data_id: Option<Arc<str>>,
    /// Human-readable top-level `inputData` name when present.
    pub name: Option<Arc<str>>,
    /// Direct nested DMN `variable` name when present.
    pub variable_name: Option<Arc<str>>,
    /// Direct nested DMN `variable` `typeRef` when present.
    pub variable_type_ref: Option<Arc<str>>,
}

impl DmnInputDataDefinition {
    /// Creates one bounded executable input-data definition.
    #[must_use]
    pub fn new(
        source_id: impl AsRef<str>,
        input_data_id: Option<impl AsRef<str>>,
        name: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            source_id: Arc::<str>::from(source_id.as_ref()),
            input_data_id: input_data_id.map(|value| Arc::<str>::from(value.as_ref())),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            variable_name: None,
            variable_type_ref: None,
        }
    }

    /// Attaches one bounded nested variable contract.
    #[must_use]
    pub fn with_variable(
        mut self,
        variable_name: Option<impl AsRef<str>>,
        variable_type_ref: Option<impl AsRef<str>>,
    ) -> Self {
        self.variable_name = variable_name.map(|value| Arc::<str>::from(value.as_ref()));
        self.variable_type_ref = variable_type_ref.map(|value| Arc::<str>::from(value.as_ref()));
        self
    }

    /// Builds one executable input-data definition from one bounded snapshot entry.
    #[must_use]
    pub fn from_snapshot(source_id: impl AsRef<str>, snapshot: &DmnInputDataSnapshot) -> Self {
        Self::new(
            source_id,
            snapshot.input_data_id.as_deref(),
            snapshot.name.as_deref(),
        )
        .with_variable(
            snapshot
                .variable
                .as_ref()
                .and_then(|variable| variable.name.as_deref()),
            snapshot
                .variable
                .as_ref()
                .and_then(|variable| variable.type_ref.as_deref()),
        )
    }
}
