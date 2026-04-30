use super::{Arc, DmnLiteralExpression};

/// One bounded executable DMN invocation parameter.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInvocationParameter {
    /// Stable parameter identifier when present in source.
    pub parameter_id: Option<Arc<str>>,
    /// Optional parameter name used by the binding.
    pub name: Option<Arc<str>>,
    /// Optional DMN `typeRef` metadata on the parameter.
    pub type_ref: Option<Arc<str>>,
}

impl DmnInvocationParameter {
    /// Creates one bounded invocation-parameter contract.
    #[must_use]
    pub fn new(
        parameter_id: Option<impl AsRef<str>>,
        name: Option<impl AsRef<str>>,
        type_ref: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            parameter_id: parameter_id.map(|value| Arc::<str>::from(value.as_ref())),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            type_ref: type_ref.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }
}

/// One bounded executable DMN invocation binding.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInvocationBinding {
    /// Stable binding identifier when present in source.
    pub binding_id: Option<Arc<str>>,
    /// Direct parameter metadata for this binding when present.
    pub parameter: Option<DmnInvocationParameter>,
    /// Direct literal-expression argument for this binding when present.
    pub argument: Option<DmnLiteralExpression>,
}

impl DmnInvocationBinding {
    /// Creates one bounded invocation-binding contract.
    #[must_use]
    pub fn new(
        binding_id: Option<impl AsRef<str>>,
        parameter: Option<DmnInvocationParameter>,
        argument: Option<DmnLiteralExpression>,
    ) -> Self {
        Self {
            binding_id: binding_id.map(|value| Arc::<str>::from(value.as_ref())),
            parameter,
            argument,
        }
    }
}

/// One bounded executable direct DMN invocation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInvocation {
    /// Stable invocation identifier when present in source.
    pub invocation_id: Option<Arc<str>>,
    /// Direct invoked-expression metadata when present.
    pub invoked_expression: Option<DmnLiteralExpression>,
    /// Direct invocation bindings preserved in source order.
    pub bindings: Vec<DmnInvocationBinding>,
}

impl DmnInvocation {
    /// Creates one bounded invocation contract.
    #[must_use]
    pub fn new(
        invocation_id: Option<impl AsRef<str>>,
        invoked_expression: Option<DmnLiteralExpression>,
        bindings: Vec<DmnInvocationBinding>,
    ) -> Self {
        Self {
            invocation_id: invocation_id.map(|value| Arc::<str>::from(value.as_ref())),
            invoked_expression,
            bindings,
        }
    }
}
