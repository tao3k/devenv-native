use jsonschema::JSONSchema;
use serde_json::Value;

pub(crate) const AUTHORING_SCHEMA: &str =
    include_str!("../../../schemas/ontology/org_ontology_authoring_contract.schema.json");
pub(crate) const TRACE_SCHEMA: &str =
    include_str!("../../../schemas/ontology/org_trace_projection_contract.schema.json");
pub(crate) const CANDIDATE_SCHEMA: &str =
    include_str!("../../../schemas/ontology/ontology_candidate_contract.schema.json");
pub(crate) const PROPERTY_SCHEMA: &str =
    include_str!("../../../schemas/ontology/org_reasoning_property_contract.schema.json");

pub(crate) fn compile_schema(raw_schema: &str) -> JSONSchema {
    let schema = match serde_json::from_str::<Value>(raw_schema) {
        Ok(schema) => schema,
        Err(error) => panic!("schema JSON must parse: {error}"),
    };
    match JSONSchema::options().compile(&schema) {
        Ok(compiled) => compiled,
        Err(error) => panic!("schema must compile: {error}"),
    }
}

pub(crate) fn assert_valid(schema: &JSONSchema, instance: &Value) {
    if let Err(errors) = schema.validate(instance) {
        let joined = errors
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        panic!("expected instance to satisfy schema: {joined}");
    }
}

pub(crate) fn assert_invalid(schema: &JSONSchema, instance: &Value) {
    assert!(
        schema.validate(instance).is_err(),
        "expected instance to violate schema"
    );
}
