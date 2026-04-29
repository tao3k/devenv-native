/// Snapshot of one direct function-definition parameter placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnFunctionDefinitionParameterSnapshot {
    /// Optional stable parameter identifier.
    pub parameter_id: Option<String>,
    /// Optional parameter name.
    pub name: Option<String>,
    /// Optional DMN `typeRef` metadata on the parameter.
    pub type_ref: Option<String>,
}

/// Snapshot of one direct function-definition literal-expression body.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnFunctionDefinitionLiteralSnapshot {
    /// Optional stable literal-expression identifier.
    pub expression_id: Option<String>,
    /// Optional DMN `typeRef` metadata on the literal expression.
    pub type_ref: Option<String>,
    /// Optional direct text payload.
    pub text: Option<String>,
}

/// Snapshot of one direct decision-owned function-definition placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnFunctionDefinitionSnapshot {
    /// Optional stable function-definition identifier.
    pub function_definition_id: Option<String>,
    /// Optional DMN function kind, for example `FEEL`.
    pub kind: Option<String>,
    /// Direct formal parameters preserved in source order.
    pub parameters: Vec<DmnFunctionDefinitionParameterSnapshot>,
    /// Direct body literal-expression metadata when present.
    pub body: Option<DmnFunctionDefinitionLiteralSnapshot>,
}
