use crate::bpmn_model_api::{BpmnInputSetSnapshot, BpmnOutputSetSnapshot};
use serde_json::{Value, json};

pub(super) fn input_set_evidence(input_set: &BpmnInputSetSnapshot) -> Value {
    json!({
        "set_id": input_set.set_id,
        "name": input_set.name,
        "data_input_refs": input_set.data_input_refs,
        "optional_input_refs": input_set.optional_input_refs,
        "while_executing_input_refs": input_set.while_executing_input_refs,
        "output_set_refs": input_set.output_set_refs,
    })
}

pub(super) fn output_set_evidence(output_set: &BpmnOutputSetSnapshot) -> Value {
    json!({
        "set_id": output_set.set_id,
        "name": output_set.name,
        "data_output_refs": output_set.data_output_refs,
        "optional_output_refs": output_set.optional_output_refs,
        "while_executing_output_refs": output_set.while_executing_output_refs,
        "input_set_refs": output_set.input_set_refs,
    })
}
