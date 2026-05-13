//! Public dmn model clause contracts for BPMN/DMN engine integration.

use serde_json::Value;
use std::sync::Arc;

/// Supported bounded DMN hit policies.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmnHitPolicy {
    /// Return the first matching rule output.
    Unique,
    /// Collect outputs from every matching rule into arrays.
    Collect,
}

/// One bounded DMN input clause.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInputClause {
    /// Stable input identifier.
    pub input_id: Arc<str>,
    /// Optional human-readable label.
    pub label: Option<Arc<str>>,
    /// Optional input name.
    pub name: Option<Arc<str>>,
    /// Optional input expression used to resolve variables.
    pub expression: Option<Arc<str>>,
    /// Optional DMN `typeRef` metadata from the executable source.
    pub type_ref: Option<Arc<str>>,
}

/// Named construction payload for one bounded DMN input clause.
#[derive(Clone, Copy)]
pub struct DmnInputClauseInput<'a> {
    /// Stable input identifier.
    pub input_id: &'a str,
    /// Optional human-readable label.
    pub label: Option<&'a str>,
    /// Optional input name.
    pub name: Option<&'a str>,
    /// Optional input expression used to resolve variables.
    pub expression: Option<&'a str>,
    /// Optional DMN `typeRef` metadata from the executable source.
    pub type_ref: Option<&'a str>,
}

impl DmnInputClause {
    /// Creates one bounded input clause.
    #[must_use]
    pub fn new(input: DmnInputClauseInput<'_>) -> Self {
        Self {
            input_id: Arc::<str>::from(input.input_id),
            label: input.label.map(Arc::<str>::from),
            name: input.name.map(Arc::<str>::from),
            expression: input.expression.map(Arc::<str>::from),
            type_ref: input.type_ref.map(Arc::<str>::from),
        }
    }

    /// Returns the preferred variable lookup path for this input clause.
    #[must_use]
    pub fn lookup_path(&self) -> Option<&str> {
        self.expression
            .as_deref()
            .filter(|value| !value.is_empty())
            .or_else(|| self.name.as_deref().filter(|value| !value.is_empty()))
            .or_else(|| self.label.as_deref().filter(|value| !value.is_empty()))
    }
}

/// One bounded DMN output clause.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnOutputClause {
    /// Stable output identifier.
    pub output_id: Arc<str>,
    /// Optional human-readable label.
    pub label: Option<Arc<str>>,
    /// Optional output name.
    pub name: Option<Arc<str>>,
    /// Optional DMN `typeRef` metadata from the executable source.
    pub type_ref: Option<Arc<str>>,
}

impl DmnOutputClause {
    /// Creates one bounded output clause.
    #[must_use]
    pub fn new(
        output_id: impl AsRef<str>,
        label: Option<impl AsRef<str>>,
        name: Option<impl AsRef<str>>,
        type_ref: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            output_id: Arc::<str>::from(output_id.as_ref()),
            label: label.map(|value| Arc::<str>::from(value.as_ref())),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            type_ref: type_ref.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }

    /// Returns the stable output key to use in evaluation results.
    #[must_use]
    pub fn output_key(&self) -> Arc<str> {
        self.name
            .as_ref()
            .filter(|value| !value.is_empty())
            .cloned()
            .or_else(|| {
                self.label
                    .as_ref()
                    .filter(|value| !value.is_empty())
                    .cloned()
            })
            .unwrap_or_else(|| Arc::clone(&self.output_id))
    }
}

/// One bounded DMN output-entry expression.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnOutputEntry {
    /// Literal output value.
    pub value: Value,
}

impl DmnOutputEntry {
    /// Creates one bounded output entry.
    #[must_use]
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}
