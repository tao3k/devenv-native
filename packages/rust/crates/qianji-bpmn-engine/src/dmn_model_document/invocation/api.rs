//! Public dmn model document invocation contracts for BPMN/DMN engine integration.

/// Snapshot of one direct invocation literal-expression placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInvocationLiteralSnapshot {
    /// Optional stable literal-expression identifier.
    pub expression_id: Option<String>,
    /// Optional DMN `typeRef` metadata on the literal expression.
    pub type_ref: Option<String>,
    /// Optional direct text payload.
    pub text: Option<String>,
}

/// Snapshot of one direct invocation parameter placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInvocationParameterSnapshot {
    /// Optional stable parameter identifier.
    pub parameter_id: Option<String>,
    /// Optional parameter name used by the binding.
    pub name: Option<String>,
    /// Optional DMN `typeRef` metadata on the parameter.
    pub type_ref: Option<String>,
}

/// Snapshot of one direct invocation binding placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInvocationBindingSnapshot {
    /// Optional stable binding identifier.
    pub binding_id: Option<String>,
    /// Direct parameter metadata when present.
    pub parameter: Option<DmnInvocationParameterSnapshot>,
    /// Direct argument literal-expression metadata when present.
    pub argument: Option<DmnInvocationLiteralSnapshot>,
}

/// Snapshot of one direct decision-owned invocation placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInvocationSnapshot {
    /// Optional stable invocation identifier.
    pub invocation_id: Option<String>,
    /// Direct invoked expression metadata when present.
    pub invoked_expression: Option<DmnInvocationLiteralSnapshot>,
    /// Direct invocation bindings preserved in source order.
    pub bindings: Vec<DmnInvocationBindingSnapshot>,
}
